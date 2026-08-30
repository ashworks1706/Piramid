# 0014 — Configuration splits by lifecycle, not by subsystem

**Context.** There was no example configuration file anywhere in the repo. `CONFIG_FILE` named a
YAML or JSON document you were expected to reconstruct from the Rust structs, and `docs/SETUP.md`
pointed at `.env.example` for "every variable" — around fifty hand-written names.

It was also two systems rather than one. `AppConfig` was file-settable; `port`, `data_dir`,
`slow_query_ms`, the disk thresholds and the whole of `EmbeddingConfig` lived in `RuntimeConfig`,
assembled in the loader from environment variables only. An embedding provider could not be
configured in a file at all. That split wasn't designed — those fields were added later, in the
loader, and stayed there.

The bug this produced is the one that decided the shape. `POST /config/reload` swapped the entire
`AppConfig`, but `current_config()` was read in exactly three places, two of which were admin
endpoints echoing the config back. Precisely one setting — the cache byte budget — did anything on
reload. `logging.level`, `slow_query_ms`, `data_dir` and the bind port were captured once in
`main.rs` and never re-read. Editing the log level and reloading returned `200` and changed
nothing.

So `AppConfig` was one object where half the fields were live and half were dead, with no way to
tell which from the outside. The obvious tidy-up — merge the env-only `ObservabilityConfig` in
`observability` into `AppConfig`, so telemetry has one home — would have made that worse: it moves
*more* start-only settings under an endpoint that claims to apply them.

**Decision.** The file has two top-level blocks, and the boundary is *when a setting takes effect*:

```yaml
startup:   # applied once at boot; changing one needs a restart
runtime:   # re-read on POST /config/reload
```

Which block a key sits in is the answer to "do I need to restart?". That is checked rather than
asserted: `AppState` keeps the startup block it booted with, and `reload_config` returns an error
if the incoming file's startup block differs, instead of accepting an edit nothing will read.

`runtime` is honest about how far it reaches. `CollectionManager` already re-read the config when
opening a collection, so a reload applies to collections opened after it and to each request that
reads a value on the way through — not to collections already resident. That is written down
rather than implied.

Everything is settable in the file, `bind`, `data_dir`, `disk` and `embedding` included.
`ObservabilityConfig` is deleted: `observability` already depends on `core`, so its settings become
`core::config::TelemetryConfig` under `startup`, and the crate keeps only its error type. Telemetry
gets one home — the outcome the tidy-up wanted, reached by putting it in the block that matches its
lifecycle rather than the one that matches its subject.

Environment variables become overrides only, spelled mechanically from the path:
`runtime.cache.max_bytes` is `PIRAMID__RUNTIME__CACHE__MAX_BYTES`. Values parse as YAML, so `8`,
`true` and `null` mean what they do in the file. This replaces about 250 lines of hand-written
parsing with one mechanism and removes the table of names that had to be kept in sync with the
structs. `OPENAI_API_KEY` stays environment-only, so a key never lands in a file that gets shared,
and the support bundle redacts it.

**One place per setting.** The audit that came with this found four ways to spell one thing:

- `IndexConfig` carried a whole `SearchConfig` in every variant, alongside `runtime.search`, so
  `ef` and `nprobe` each had two homes. Dropped.
- The per-family parameter structs carried `mode` alongside `runtime.execution`. Now `serde(skip)`,
  set from `runtime.execution` when the index is built.
- Thread count had three spellings: `hardware.cpu_threads`, `parallelism.mode: Fixed(n)`, and
  `NUM_THREADS`. One remains, `startup.threads`, `null` meaning one per core.
- `hardware.gpu_enabled` was a second switch that could disagree with `hardware.profile`. It is now
  derived from the profile.

`HardwareProfile`'s memory presets are deleted rather than fixed. `Memory8Gb` and its siblings
silently overwrote `cache.max_bytes` and two other values a user may have set explicitly — the same
defect as a backend that quietly serves different numbers. A profile now names hardware
(`auto | cpu-only | gpu`) and nothing else.

**Nothing is silently ignored.** `deny_unknown_fields` throughout, so a misspelling or a key in the
wrong block fails at startup naming it.

Settings whose implementation doesn't exist yet are present rather than absent — `runtime.inference`
with its paged `kv_cache`, sampling and `augment` sections, and `startup.embedding.provider:
piramid` — so the v0.4.0 work lands into a settled shape instead of retrofitting one. `validate`
refuses them at startup with the roadmap version in the message. This is what `QuantizationLevel`
already did for `Int4` and `Float16`; the rule is that a knob may exist before its code, but it may
never accept a value nothing reads.

**The example is tested.** `config.example.yaml` is the whole surface at its defaults. One test
asserts it deserializes to exactly `Config::default()`; another asserts every key of the serialized
default appears in it. It cannot drift into being wrong, which is the failure mode of every
configuration document that is merely written.

**Consequences.** A wire-format break: enum values are lowercase (`type: hnsw`, `metric: cosine`,
`level: none`) and `IndexConfig` no longer carries `search`. Persisted collection configs written
by an older build will not parse. There is no migration, because there is no data to migrate —
taken deliberately now, since the same break costs a great deal more once anyone is running this.

`.env.example` shrinks from about fifty names to three plus the override pattern.
`init_rayon_pool`, which ran on every collection open and swallowed the already-built error, moves
to the binary and runs once from `startup.threads`.

Not done: `runtime` still means "collections opened after the reload". Making it per-request is on
the roadmap, and until then the reach is documented rather than overstated.
