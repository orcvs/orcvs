# 08 — Document Built-in Synth CCs and MIDI messages

**What to build:** A reader sending Control Change, Pitch Bend or Monophonic Play knows what each
does on external gear and on the Built-in Synth, and the ADR wording about the acid voice
describes what it does.

**Blocked by:** 07 (the Built-in Synth table), 14 (the controller map), 15 (the drum keys), 18
(`legato`).

**Status:** ready-for-agent

- [ ] The reference has a table of the Built-in Synth's effect controllers as ticket 14 settles
      them, each with its General MIDI name: CC1 vibrato, CC72 `LN`, CC73 `OF`, CC74 `EQ`, CC91
      `R1`, CC92 `R2`, CC93 `CH`, CC94 `DL`.
- [ ] It explains that reverb, chorus and echo are per-channel sends, set by sending the controller
      on the channel whose send it changes, and that the other effects apply to the whole mix.
- [ ] The Control Change example sets an effect with its new controller and states what it does.
- [ ] Pitch Bend's entry states the 14-bit value (`msb × 128 + lsb`, 0–16383, centre 8192) and
      that the receiver sets the bend range.
- [ ] The reference states that the Built-in Synth ignores Pitch Bend and Program Change, that
      Reset All Controllers restores the acid voice's controls, and that All Sound Off releases
      notes rather than cutting them.
- [ ] The acid voice's documentation states that accent is fixed (from velocity) and that it has no
      tuning control.
- [ ] Monophonic Play's entry states that it retriggers by default, and that writing `legato` makes
      notes overlap so the acid voice slides; overlapping Timed Plays also slide.
- [ ] The reference lists the kit's General MIDI keys as ticket 15 assigns them, and says other
      keys are silent.
- [ ] ADR 0099 and the acid voice's documentation say "slide" for the 303 behaviour throughout.
- [ ] `cargo nextest run --package orcvs --locked` passes (it checks every example, summary and
      heading in the Function reference), and `mise run site_build` and `mise run site_test` pass.

## Comments

Source: `audit.md`, top findings 4, 5 and 9, and Part 3; the controller map, keys and `legato`
come from `decisions.md`.
