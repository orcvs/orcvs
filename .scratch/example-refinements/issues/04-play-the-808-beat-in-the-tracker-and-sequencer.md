# 04 — Play the 808 beat in the Tracker and the Sequencer

Status: resolved

Both ship with their steps nearly empty, so opening one plays almost nothing. Ship each playing
the basic 4/4 808 beat the 808 example plays, so it shows itself working.

## Acceptance criteria

- [x] The Tracker's first three voices play kick `C2`, snare `D2` and closed hat `f2` on channel
      `08`: kick on steps 0, 4, 8 and 12, snare on 4 and 12, hat on every even step.
- [x] The Sequencer's first three voices play the same beat over both of its bars.
- [x] Every other voice ships empty and rests silently.
- [x] The guides describe the beat and the channel.

## Comments

### 2026-10-29 — Resolved

The first three voices of each, Learn Orca's channels `00` to `02`, now play on channel `08` and
hold the beat as Comment text: kick `C2`, snare `D2` and hat `f2`, all at the shared velocity `77`.
The kit picks each instrument from the Note's pitch class, so the three share the channel as the
808's three Timed Plays do. The Playback tests run both shipped files for two loops against the
beat; the per-voice tests, the live-editing test and the loopback blank the beat first and place
Notes by voice rather than by channel. The live-editing test edits channel `03`'s empty voice.
