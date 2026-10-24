# 06 — Install the cargo tools as release binaries

**What to build:** A CI job that installs nextest, cargo-deny, trunk and wasm-pack in seconds when the mise tool cache misses, by downloading each tool's pinned release binary instead of compiling it.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] The four tools are installed from their upstream release binaries at the same pinned versions; no `cargo:` backend remains in `[tools]`.
- [x] The tooling contract pins each tool at an exact version under its new backend.
- [ ] A pull request that edits `mise.toml` spends seconds, not minutes, in the mise install step.

## Comments

### Why

Every `cargo:` tool in `mise.toml` is compiled from source whenever the mise cache misses. A
pull request that edits `mise.toml` always misses, because the key hashes the file, and pull
request runs never save a cache. On #289 the install step took 7 and 11.5 minutes on `full-gate`
and 12 minutes on `wasm`. Combined with a cold cargo cache, that pushed `full-gate` past its
twenty-minute bound. Warm, the checks themselves take about ninety seconds.

All four tools publish release binaries for Linux and macOS at the pinned versions:
`nextest-rs/nextest` (`cargo-nextest-0.9.137`), `EmbarkStudios/cargo-deny` (`0.20.2`),
`trunk-rs/trunk` (`v0.21.14`) and `rustwasm/wasm-pack` (`v0.15.0`). `actionlint` and `zizmor`
already install through mise's `aqua:` backend.
