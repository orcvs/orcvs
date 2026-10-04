# 06 — Deliver notes and rests without replaying stale output

Status: needs-triage
Blocked by: 05

## Problem

An absent read can leave the previous Note in a destination. If a periodic Bang
still activates Timed Play, a visually empty step can sound that old Note.

## Work

Apply the accepted rest representation through selection, activation and MIDI
delivery. Define whether a rest only omits a new onset or also terminates an
existing voice, as decided in 01–02; do not accidentally infer either behavior
from how long the preceding note happens to last.

## Acceptance

- [ ] Note–rest–note, repeated notes, consecutive rests, first/last-step rests,
      all-rest patterns and wraparound emit the exact expected command timeline.
- [ ] A rest never retriggers a surviving Note operand, and displayed `**` never
      reactivates MIDI on the following Tick.
- [ ] Invalid or half-edited selected data has the accepted diagnostic and emits
      no spurious onset. Fixing it resumes on the next eligible Tick.
- [ ] Playback tests use the existing in-memory output adapter and controlled
      time to assert Note On/Off bytes, channel, velocity and expiry, including
      a prior note whose duration extends over a rest.
- [ ] Stop/restart and output replacement preserve existing safety/lifecycle
      behavior; no new scheduler or hidden voice state is added without a decision.

The ordinary language Tick test proves Play Commands. This ticket additionally
proves what Playback delivers and when, so requested length is not confused
with an observed Note Off.
