# Security

## Reporting

Report vulnerabilities through
[GitHub Security Advisories](https://github.com/ashworks1706/piramid/security/advisories/new).
Please do not open a public issue for something exploitable.

Include what an attacker gains, how to reproduce it, and the affected version. Expect an
acknowledgement within a week.

## Supported versions

Pre-1.0: only the latest release gets fixes.

## Threat model — read this before deploying

**Piramid has no authentication, no authorization, and no rate limiting.** Any client that can
reach the port can read, write, and delete every collection.

It is built to run on a trusted network — localhost, a private subnet, or behind a gateway that
terminates auth. Do not expose port 6333 to the internet.

Specifically:

- **CORS is wide open** (`allow_origin(Any)`). Any web page can call the API from a browser.
  Combined with no auth, a user visiting a hostile page while a local Piramid is running can have
  their data read or destroyed. Restrict this in your reverse proxy.
- **No transport encryption.** Terminate TLS upstream.
- **No tenant isolation.** Collections are not a security boundary.
- **The body limit is 100 MB** and there is no request-rate limit, so an unauthenticated caller can
  exhaust memory or disk. `DISK_MIN_FREE_BYTES` bounds the disk case only.

## Handling secrets

`OPENAI_API_KEY` and other provider credentials come from the environment and belong in `.env`,
which is gitignored. They are never logged. Do not put them in a compose file or an image.

## Dependencies

`cargo deny check advisories bans licenses sources` runs in CI weekly and on every PR
(`just audit`). Dependabot proposes updates weekly.

## Unsafe code

`unsafe_code` is denied workspace-wide. It is permitted in `crates/gpu` (device memory) and at
exactly two audited sites — `storage::persistence::mmap::create_mmap` and `server::runtime::disk`
— each carrying a `// SAFETY:` comment stating its precondition. A PR introducing `unsafe`
anywhere else will not pass CI.
