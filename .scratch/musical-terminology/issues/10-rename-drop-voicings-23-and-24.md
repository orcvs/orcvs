# 10 — Rename the drop 2&3 and drop 2&4 Voicings

**What to build:** A musician writes drop 2&3 as `23` and drop 2&4 as `24`, matching how jazz
texts name them, instead of the digit-sum names `D5` and `D6`.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] `D5` → `23` and `D6` → `24`; `CL`, `OP`, `D2`, `D3`, `SH` and `RL` are unchanged.
- [ ] The Rust variants are named for the dropped positions (drop two and three, drop two and
      four), and the digit-sum explanation is gone from code and docs.
- [ ] The uniqueness check across Interval Sets, Voicings and Functions still holds.
- [ ] ADR 0105 (and ADR 0103 where it names the drops) is amended.
- [ ] The glossary, the Function reference, examples and tests use only the new names.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package lang --all-targets --locked -- -D
      warnings`, `cargo clippy --package orcvs --all-targets --locked -- -D warnings`,
      `cargo nextest run --package lang --locked` and `cargo nextest run --package orcvs --locked`
      pass, and `mise run site_build` and `mise run site_test` pass.

## Comments

Decided in `decisions.md`.
