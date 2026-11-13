# 05 — State that a Tick is a sixteenth note

**What to build:** A reader of the Function reference can turn any Tick count into beats: a Tick
is a sixteenth note, four to a beat at the configured BPM, so a Play's `length` of `04` is one
beat.

**Blocked by:** 11 (docs describe the names and behaviour those tickets settle).

**Status:** ready-for-agent

- [ ] The Function reference's "How to read an entry" (or equivalent) states the Tick's musical
      length and that it follows the BPM.
- [ ] The glossary's Tick and BPM entries state the same ratio.
- [ ] Each Play's `length` is glossed as the note's gate length in Ticks.
- [ ] The reference distinguishes a Play's `length` from the period `length` of Clock and Delay
      and from the Sound Module's `LN` effect.
- [ ] `cargo nextest run --package orcvs --locked` passes (it checks every example, summary and
      heading in the Function reference), and `mise run site_build` and `mise run site_test` pass.

## Comments

Source: `audit.md`, top findings 8 and 10, and Part 2.
