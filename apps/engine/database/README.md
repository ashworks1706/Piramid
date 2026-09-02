# piramid-database

Where vectors live on disk: records, the write-ahead log, mmap, sidecar files, the offset index and
the manifest.

Decides nothing about API behaviour or collection lifecycle — a `RecordStore` does not know what a
collection is.

`vectors::VectorReader` is how indexes read vectors they don't own. `as_slab()` is the fast path
and `gather_into()` the fallback; both have defaults, so a new reader costs nothing.

`unsafe` appears once, at `storage::persistence::mmap::create_mmap`, with a `// SAFETY:` note.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) for how the crates fit together.
