# 08 — Deliver the editable-cell tracker and acceptance evidence

Status: needs-triage
Blocked by: 07

## Work

Ship a small Source File and walkthrough that demonstrate the requested tracker
interaction. Use a clock-indexed row of note data and deliberate rests. If the
current constructed-Sequence experiment remains in the branch, replace it or
label it accurately so it is not offered as evidence for cell reading.

## Acceptance

- [ ] The checked-in Source uses the accepted grammar with one editable Note
      per occupied step. It runs from the ordinary console File > Open flow.
- [ ] The guide explains the index, extent, Note row, trigger, rest behavior,
      BPM, MIDI channel, velocity and duration, with exact coordinates where
      alignment matters. It distinguishes Orca's source syntax from Orcvs's.
- [ ] The test loads this exact Source File, checks at least two complete loops,
      edits a future step, turns another into a rest, and verifies subsequent
      Source and Playback results. The program is not duplicated in a test string.
- [ ] Instructions cover changing a Note, inserting a rest, changing the rate
      and extending the pattern using the accepted extent contract.
- [ ] A manual run records the MIDI output device, instrument/channel, BPM and
      observed note/rest/edit behavior. Unavailable hardware is reported as
      unverified; an in-memory adapter does not count as an audible test.
- [ ] Required scoped checks, Function reference checks and all triggered gates
      pass; remaining CI-only coverage is stated. The whole diff is reviewed.
- [ ] ADRs, `CONTEXT.md`, the capability map and this effort's tickets agree on
      what shipped and what remains deferred. No reserved operation is described
      as implemented merely because this example would benefit from it.

Resolve this ticket only when both the program and its editing workflow meet the
definition of done in `../spec.md`.
