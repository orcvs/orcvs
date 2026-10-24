# 09 — Verify each tool download against a recorded checksum

**What to build:** Every tool `mise.toml` pins installs only if its download matches a checksum committed to the repository, on each platform the gates run on (Linux x64 in CI, macOS arm64 in the merge queue and locally).

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] A committed lockfile records the download and checksum of every `[tools]` pin for Linux x64 and macOS arm64, and mise refuses an install whose download does not match.
- [ ] CI installs through that lockfile, so a replaced release asset fails the install step rather than running.
- [ ] The tooling contract fails when a `[tools]` pin has no matching lockfile entry, so a version bump that skips regenerating the lockfile is rejected.
- [ ] `docs/tooling.md` states the bump procedure: edit the pin, regenerate the lockfile, commit both.

## Comments

### Why

Issue 06 moved trunk and wasm-pack to mise's `github:` backend, which downloads the release asset
over HTTPS from a pinned tag but records no checksum. If an upstream release asset were replaced,
CI would run the replacement unnoticed. The `aqua:` pins (nextest, cargo-deny, actionlint, zizmor)
are checked against the aqua registry's checksums, and `node` against its own; a lockfile puts all
of them under one recorded hash the repository controls.

The exposure is a compromised upstream release, not a mistake here: the versions are pinned, and
the pull-request jobs that run these tools hold read-only tokens and no secrets. The cost is one
extra step per tool bump, which the contract enforces.

Raised in review of #295; Toby asked for it as a follow-up (2026-10-24).
