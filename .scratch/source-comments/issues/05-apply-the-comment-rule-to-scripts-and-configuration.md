# 05 — Apply the comment rule to scripts and configuration

**What to build:** Every `#` and `//` comment in `scripts/`, `mise.toml`, the `Cargo.toml` files, `deny.toml` and `.github/workflows/` held to `docs/agents/comments.md`. `scripts/check-tooling-contract.sh` is most of it: its comments explain why each assertion exists, and several do it through history (for example, the exact egui pin's "which is how this workspace once resolved 0.36.2 under a `0.36.1` requirement", at `scripts/check-tooling-contract.sh:630-631`). `orcvs/Cargo.toml:50-51` explains a removed dependency through "a retired clock task". Comments only.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] No comment in scope narrates lineage except as the thing a present-tense constraint forbids
- [ ] Each remaining citation names a constraint the configuration cannot show
- [ ] `bash scripts/check-tooling-contract.sh`, `actionlint` and `zizmor --offline .github/workflows` pass

## Comments

### Audit at cad296df — 2026-09-29

Blocker `01` is resolved, so `ready-for-agent` stands. The lineage comments are still there; the
body now names three concrete sites: `scripts/check-tooling-contract.sh:630-631` (the egui pin),
its duplicate at `console/Cargo.toml:85-86`, and `orcvs/Cargo.toml:50-51` ("a retired clock task").
Other candidates in `scripts/check-tooling-contract.sh` include `:258` ("used to live on the
merge"). No criterion is met; the gates were not run for this audit.

### Issue audit against d3fd1b27 — 2026-10-01

The `console/Cargo.toml` duplicate was removed by 00280c39 (#182); the body no longer names it.
Still open: `scripts/check-tooling-contract.sh:630-631` and `:258`, further candidates at `:308`,
`:357`, `:376` and `:387`, and `orcvs/Cargo.toml:50-51`.
