# 09 — Verify each tool download against a recorded checksum

**What to build:** Every tool `mise.toml` pins installs only if its download matches a checksum committed to the repository, on each supported platform: Linux x64 (CI) and macOS arm64 (the merge queue and local development). On any other platform the install fails loudly rather than falling back to an unverified download.

**Blocked by:** None — can start immediately.

**Related:** `verification-gaps/13` (bump the mise tool pins automatically). Once the lockfile is mandatory, whatever bumps a pin must also regenerate the lockfile, which rules out a mechanism that only rewrites `mise.toml`. Decide the two together.

**Status:** ready-for-agent

- [ ] A committed lockfile records the version, download and checksum of every `[tools]` pin for Linux x64 and macOS arm64.
- [ ] Every install that reads `mise.toml`, in CI and locally, runs in mise's locked mode, set in `mise.toml` or the workflow environment rather than per command, so a download with no matching entry or a mismatched checksum fails the install. A test shows that a missing platform entry fails it.
- [ ] The runners are pinned to the supported architectures, or the workflows assert them, so a runner image that changes architecture fails rather than installing unverified.
- [ ] The tooling contract fails when a `[tools]` pin's version differs from its lockfile entry, or the entry lacks Linux x64 or macOS arm64, so a bump that skips regenerating the lockfile is rejected.
- [ ] `docs/tooling.md` states the bump procedure (edit the pin, regenerate the lockfile, commit both) and the supported platforms.

## Comments

### Why

Issue 06 moved trunk and wasm-pack to mise's `github:` backend, which downloads the release asset
over HTTPS from a pinned tag but records no checksum. If an upstream release asset were replaced,
CI would run the replacement unnoticed. Whether mise already verifies
the `aqua:` pins and `node` against upstream checksums is unverified here; the lockfile covers all
seven pins either way, so the criteria do not depend on it.

The exposure is a compromised upstream release, not a mistake here: the versions are pinned, and
the pull-request jobs that run these tools hold read-only tokens and no secrets. The cost is one
extra step per tool bump, which the contract enforces.

Raised in review of #295; Toby asked for it as a follow-up (2026-10-24).
