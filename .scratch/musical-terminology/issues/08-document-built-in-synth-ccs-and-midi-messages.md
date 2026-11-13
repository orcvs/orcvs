# 08 — Document Built-in Synth CCs and MIDI messages

**What to build:** A reader sending Control Change, Pitch Bend or Monophonic Play knows what each
does on external gear and on the Built-in Synth, and the code comments about the Built-in Synth
describe what it does.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] The reference warns that the Built-in Synth's effect controllers CC 1–8 differ from General
      MIDI (CC5 portamento time, CC6 data entry, CC7 channel volume, CC8 balance) and apply to
      every channel.
- [ ] The Control Change example is annotated, or replaced, so it no longer reads as a reverb
      level that would halve the volume on external gear.
- [ ] Pitch Bend's entry states the 14-bit value (`msb × 128 + lsb`, 0–16383, centre 8192) and
      that the receiver sets the bend range.
- [ ] The reference states that the Built-in Synth ignores Pitch Bend and Program Change.
- [ ] Monophonic Play's entry states that it retriggers (Note Off before each Note On) and so never
      slides, and that a slide on the acid voice comes from overlapping Timed Plays.
- [ ] The drum kit's code comment no longer claims General MIDI order, and states the actual
      pitch-class mapping or its differences from GM.
- [ ] `cargo fmt --all -- --check` and the clippy and nextest gates for `synth` pass;
      `mise run site_build` and `mise run site_test` pass.

## Comments

Source: `audit.md`, top findings 4, 5 and 9, and Part 3. Renaming the `Glide` smoother and
changing the CC map are out of scope.
