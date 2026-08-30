# Security

## Reporting

Report vulnerabilities through
[GitHub Security Advisories](https://github.com/ashworks1706/piramid/security/advisories/new).
Please don't open a public issue for something exploitable.

Include what an attacker gains, how to reproduce it, and the affected version. Expect an
acknowledgement within a week.

## Supported versions

Pre-1.0, so only the latest release gets fixes.

## Threat model

Piramid has no authentication, no authorization, and no rate limiting. Any client that can reach
the port can read, write, and delete every collection.

It's built to run on a trusted network: localhost, a private subnet, or behind a gateway that
terminates auth. Don't expose port 6333 to the internet.

Specifically:

- CORS is wide open (`allow_origin(Any)`), so any web page can call the API from a browser.
  Combined with no auth, someone visiting a hostile page while a local Piramid is running can have
  their data read or destroyed. Restrict this in your reverse proxy.
- There's no transport encryption. Terminate TLS upstream.
- Collections are not a security boundary.
- The body limit is 100 MB and there's no request rate limit, so an unauthenticated caller can
  exhaust memory or disk. `DISK_MIN_FREE_BYTES` bounds the disk case only.

## Telemetry

Nothing is transmitted to this project under any configuration. The exporters in
`piramid-observability` point at endpoints you supply, and `PIRAMID_LOG_SPANS` only writes to your
own logs.

Span fields carry collection names and request ids. They never carry vector contents, document
text, or metadata values.

## Diagnostic bundles

`piramid support-bundle` writes version, platform, build features, resolved configuration, and
collection state to a file for attaching to a bug report. Variables whose names look like
credentials — containing `KEY`, `TOKEN`, `SECRET`, `PASSWORD`, `DSN`, `CREDENTIAL`, or `AUTH` —
are reported as `<redacted, N chars>`, since whether a key is set is diagnostic but its value
never is.

The bundle still contains your configuration and collection names. Read it before sharing.

## Handling secrets

`OPENAI_API_KEY` and other provider credentials come from the environment and belong in `.env`,
which is gitignored. They're never logged. Don't put them in a compose file or an image.

## Dependencies

`cargo deny check advisories bans licenses sources` runs in CI weekly and on every PR, or locally
as `just audit`. Dependabot proposes updates weekly.

## Unsafe code

`unsafe_code` is denied workspace-wide. It's permitted in `apps/engine/hardware/gpu` for device
memory, and at two audited sites: `storage::persistence::mmap::create_mmap` and
`server::disk`. Each carries a `// SAFETY:` comment stating its precondition. A PR
introducing `unsafe` anywhere else fails CI.
