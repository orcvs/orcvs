# 05 — Rest a Chord Play on a blank root

Status: needs-triage

Blocked by: 01, 03

**What to build:** ADR 0104 for both Chord Plays. A fully blank root is a Rest by any route, and it
ignores other blank or cleared operands, so a wholly empty pattern row rests.

## Acceptance criteria

- [ ] Both Chord Plays rest on a blank root by each of the three routes, sending nothing and diagnosing nothing.
- [ ] A blank root with blank or cleared channel, velocity, set, voicing, inversion or length is still a Rest, and a Rest checks no operand's range.
- [ ] A half-written operand beside a blank root is diagnosed.
- [ ] A blank Interval Set with a root present is diagnosed.
- [ ] The Language Map admits a Chord Play with a blank root as a root.
- [ ] `CONTEXT.md`'s Rest entry covers Chord Play.

## Comments

- ADR 0108 supersedes ADR 0104, which this ticket implements, and removes the Rest. A Play's note
  is optional by its operand type, `Optional<Note>`, and a Chord Play's root stays a plain Note.
  Whether an empty pattern row leaves a Chord Play silent, by making its root, Interval Set,
  Voicing and inversion optional, is its own decision (ADR 0108, Consequences). Rewrite this ticket
  in those terms, or close it as `wontfix`, before an agent picks it up.
