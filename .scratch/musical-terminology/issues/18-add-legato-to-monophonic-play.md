# 18 — Add legato to Monophonic Play

**What to build:** A musician can make a run of Monophonic Play notes slide on the acid voice, or
play legato on an external mono synth, by writing a `legato` operand, set per note from Source.

**Blocked by:** 17.

**Status:** ready-for-agent

- [ ] `!% channel velocity note length legato`: `legato` is a Number that defaults to `00` when
      blank.
- [ ] `00` retriggers as today (Note Off before Note On); any other value sends the new Note On
      before the old Note Off.
- [ ] A legato note at the same pitch as the note it replaces keeps that note sounding until the
      new note's expiry, and sends no Note Off for it in between; a test plays two consecutive
      legato C4s and checks the second is not cut short.
- [ ] On the Built-in Synth's acid voice, a legato note slides to its pitch.
- [ ] The Function reference documents `legato`, and says that on the acid voice it means slide.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package lang --all-targets --locked -- -D
      warnings`, `cargo clippy --package orcvs --all-targets --locked -- -D warnings`,
      `cargo nextest run --package lang --locked` and `cargo nextest run --package orcvs --locked`
      pass, and `mise run site_build` and `mise run site_test` pass.

## Comments

Decided in `decisions.md`.
