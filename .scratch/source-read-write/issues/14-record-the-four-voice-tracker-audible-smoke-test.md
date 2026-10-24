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

### 2026-10-18

Open the tracker with Help > Examples > Orca Tracker, choose a MIDI output in the Panel, and
press Space.

### 2026-10-18 — The tracker is now Learn Orca's

`examples/tracker.orcvs` is now a port of the Learn Orca tracker: eight voices on channels `00` to
`07`, sixteen steps, one step a Tick. Record what each channel's steps sounded like against
`examples/tracker.md`.
