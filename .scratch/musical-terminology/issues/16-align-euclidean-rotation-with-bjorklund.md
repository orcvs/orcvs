# 16 — Align Euclidean rotation with Bjorklund

**What to build:** `~% hits steps` produces the same pattern as the published Bjorklund tables,
so a rhythm copied from another tool sounds the same in Orcvs.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] For every `hits` ≤ `steps` ≤ 32, the pattern equals the Bjorklund pattern starting on an
      onset (for example E(2,5) is `x.x..` and E(5,8) is `x.xx.xx.`), checked by a test against a
      reference implementation or table.
- [ ] The Euclidean ADR and the Function reference describe the Bjorklund rotation.
- [ ] Examples and tests whose patterns change are updated.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package lang --all-targets --locked -- -D
      warnings`, `cargo clippy --package orcvs --all-targets --locked -- -D warnings`,
      `cargo nextest run --package lang --locked` and `cargo nextest run --package orcvs --locked`
      pass, and `mise run site_build` and `mise run site_test` pass.

## Comments

Decided in `decisions.md`. The audit found about 42% of patterns rotated differently today.
