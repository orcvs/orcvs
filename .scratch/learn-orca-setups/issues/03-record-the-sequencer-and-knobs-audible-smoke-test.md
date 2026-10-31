# 03 — Record the sequencer and knobs audible smoke test

Status: ready-for-human
Blocked by: 01, 02

**What to build:**

Play `examples/sequencer.orcvs` and `examples/knobs.orcvs` through a physical MIDI device and record
what was heard and seen. Automated evidence alone does not complete either example's delivery.

## Acceptance criteria

- [ ] With a Note typed into some steps of several voices, record that each sounded on its own
      channel at its step and that empty steps rested, with the device and the BPM.
- [ ] Record that each knob's controller `01` to `08` moved to its value in a preset, and glided to
      the next when the preset changed, with the device and the BPM (see `04`).

## Comments
