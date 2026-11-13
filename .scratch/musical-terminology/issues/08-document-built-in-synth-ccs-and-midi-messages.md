# 08 — Document Built-in Synth CCs and MIDI messages

**What to build:** A reader sending Control Change, Pitch Bend or Monophonic Play knows what each
does on external gear and on the Built-in Synth, and the comments and ADR wording about the
Built-in Synth describe what it does.

**Blocked by:** 07 (it owns the Built-in Synth table this ticket's warnings sit beside).

**Status:** ready-for-agent

- [ ] The reference lists each effect controller whose Built-in Synth meaning differs from General
      MIDI, with the GM meaning beside it: CC2 (breath), CC4 (foot), CC5 (portamento time), CC6
      (data entry), CC7 (channel volume) and CC8 (balance), and CC3's `LN`. CC1, applied as
      vibrato, matches GM modulation and is not listed as a clash. It states that these effects
      apply on every channel.
- [ ] The Control Change example is annotated, or replaced, so it no longer reads as a reverb
      level that would halve the volume on external gear.
- [ ] Pitch Bend's entry states the 14-bit value (`msb × 128 + lsb`, 0–16383, centre 8192) and
      that the receiver sets the bend range.
- [ ] The reference states that the Built-in Synth ignores Pitch Bend and Program Change, that
      Reset All Controllers restores the acid voice's controls, and that All Sound Off releases
      notes rather than cutting them.
- [ ] The acid voice's documentation states that accent is fixed (from velocity) and that it has no
      tuning control.
- [ ] Monophonic Play's entry states that it retriggers (Note Off before each Note On) and so never
      slides, and that a slide on the acid voice comes from overlapping Timed Plays.
- [ ] ADR 0099 and the acid voice's documentation say "slide" for the 303 behaviour throughout.
- [ ] The drum kit's code comment states the actual pitch-class mapping, in the present tense, per
      `docs/agents/comments.md`, rather than claiming General MIDI order.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package synth --all-targets --locked -- -D
      warnings` and `cargo nextest run --package synth --locked` pass.
- [ ] `cargo nextest run --package orcvs --locked` passes (it checks every example, summary and
      heading in the Function reference), and `mise run site_build` and `mise run site_test` pass.

## Comments

Source: `audit.md`, top findings 4, 5 and 9, and Part 3. Renaming the `Glide` smoother and
changing the CC map or the reset behaviour are out of scope.
