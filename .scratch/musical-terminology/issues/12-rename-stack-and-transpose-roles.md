# 12 — Rename Stack's and Up/Down's Roles

**What to build:** Stack's position operand reads `index`, zero-based like Pick's, and Up's and
Down's distance operand reads `semitones`.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] Stack's `degree` Role is `index`; Up's and Down's `number` Role is `semitones`.
- [ ] Info, the Function reference signatures and the glossary use the new Roles.
- [ ] Behaviour is unchanged.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package lang --all-targets --locked -- -D
      warnings`, `cargo clippy --package orcvs --all-targets --locked -- -D warnings`,
      `cargo nextest run --package lang --locked` and `cargo nextest run --package orcvs --locked`
      pass, and `mise run site_build` and `mise run site_test` pass.

## Comments

Decided in `decisions.md`.
