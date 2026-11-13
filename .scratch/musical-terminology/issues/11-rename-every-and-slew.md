# 11 — Rename Delay to Every and Interpolation to Slew

**What to build:** The timing Functions read as a musician expects: `~*` is **Every** (a clock
divider), `~>` is **Slew** (a slew limiter), and the period operand of Clock and Every is `rate`,
so `length` only ever means a note's gate length.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] The Function table names `~*` Every and `~>` Slew; spellings and behaviour are unchanged.
- [ ] Clock's and Every's `length` Role is renamed `rate`.
- [ ] Info, the Function reference, the glossary, the console's Function Reference and the site
      show the new names and Role.
- [ ] The timing research notes drop the separate `~/` Every proposal and describe `~*` as Every.
- [ ] The Sound Module's echo effect keeps the name Delay.
- [ ] The ADRs that name Delay or Interpolation as current vocabulary are amended.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package lang --all-targets --locked -- -D
      warnings`, `cargo clippy --package orcvs --all-targets --locked -- -D warnings`,
      `cargo nextest run --package lang --locked` and `cargo nextest run --package orcvs --locked`
      pass, and `mise run site_build` and `mise run site_test` pass.

## Comments

Decided in `decisions.md`.
