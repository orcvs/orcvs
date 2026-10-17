# 13 — Record the cell tracker audible smoke test

Status: ready-for-human
Blocked by: 06

**What to build:**

Play the shipped cell tracker example from ticket 06 through a physical MIDI device and record what
was heard. Automated evidence alone does not complete the example's delivery.

## Acceptance criteria

- [ ] Record an audible MIDI smoke test of the shipped tracker Source File, with the device, MIDI
      channel and BPM used.
- [ ] The record states whether the occupied pairs sounded, whether the empty pairs left rests, and
      whether notes sustained through later empty pairs as the guide describes.

## Comments

### Decided 2026-10-13 by Toby

Split from ticket 06's last criterion, which needs a person with a MIDI device, so 06 can stay
`ready-for-agent` for its automated work.

### 2026-10-17

Unblocked: `06` shipped `examples/cell-tracker.orcvs`. Open it, choose a MIDI output, and press
Space; the guide's tempo, channel and layout are in `examples/cell-tracker.md`.
