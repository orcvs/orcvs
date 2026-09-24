# 05 — Apply the comment rule to scripts and configuration

**What to build:** Every `#` and `//` comment in `scripts/`, `mise.toml`, the `Cargo.toml` files, `deny.toml` and `.github/workflows/` held to `docs/agents/comments.md`. `scripts/check-tooling-contract.sh` is most of it: its comments explain why each assertion exists, and several do it through history (for example, the exact egui pin's "which is how this workspace once resolved 0.36.2 under a `0.36.1` requirement"). Comments only.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] No comment in scope narrates lineage except as the thing a present-tense constraint forbids
- [ ] Each remaining citation names a constraint the configuration cannot show
- [ ] `bash scripts/check-tooling-contract.sh`, `actionlint` and `zizmor --offline .github/workflows` pass
