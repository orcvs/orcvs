# 07 — Explain MIDI channel numbering

**What to build:** A musician routing Orcvs to a DAW or to the Built-in Synth knows which channel
they will see: `00`–`0F` are MIDI channels 1–16, and the reference shows which channels the
Built-in Synth assigns to which voice.

**Blocked by:** None — can start immediately. Owns the Built-in Synth channel table.

**Status:** ready-for-agent

- [ ] The Function reference's channel operand states that `00` is MIDI channel 1 and `0F` is 16.
- [ ] The glossary's channel wording says the same.
- [ ] The reference (or the Built-in Synth's documentation) has a table of the Built-in Synth's
      channels and voices, in both spellings, taken from the synth itself: tones on `00`–`07`,
      the acid voice on `09`, the kit on `08` and `0A`–`0F`.
- [ ] It warns that the acid voice's `09` is MIDI channel 10, the General MIDI drum channel, so a
      GM drum pattern sent there plays the bassline.
- [ ] The drum module's doc comment states the kit's channels correctly (it does not include `9`).
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package synth --all-targets --locked -- -D
      warnings` and `cargo nextest run --package synth --locked` pass.
- [ ] `cargo nextest run --package orcvs --locked` passes (it checks every example, summary and
      heading in the Function reference), and `mise run site_build` and `mise run site_test` pass.

## Comments

Source: `audit.md`, top finding 3 and Part 3.
