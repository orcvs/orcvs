# 14 — Record the four-voice tracker audible smoke test

Status: ready-for-human
Blocked by: 03

**What to build:**

Play `examples/tracker.orcvs` through a physical MIDI device and record what was heard. Automated
evidence alone does not complete the example's delivery.

## Acceptance criteria

- [ ] Record an audible MIDI smoke test of the shipped tracker, with the device, the MIDI channels
      used for each voice and the BPM.
- [ ] The record states whether every voice's occupied steps sounded, whether empty steps left
      rests, and whether notes sustained through an empty step after them as `examples/tracker.md`
      describes.

## Comments

Split from `03`'s last criterion on 2026-10-17, as `cell-tracker/13` was split from
`cell-tracker/06`.
