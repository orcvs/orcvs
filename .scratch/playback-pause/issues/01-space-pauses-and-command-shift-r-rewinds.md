# 01: Space pauses and resumes Playback; Command Shift R rewinds to Tick 0

**What to build:** A stop holds the absolute Tick, and the next start resumes
at the Tick after the last one executed. A Rewind makes Tick `0` the next Tick,
bound to Command Shift R on native. See `../spec.md` and ADR 0076.

**Blocked by:** None (can start immediately)

**Status:** resolved

- [x] `PlaybackInner::begin_run` keeps the absolute Tick and counts the Run
      Clock on from the held value.
- [x] `PlaybackEngine::rewind` and a Rewind slot in the mailbox, applied after
      the stop: a stopped engine is rewound, a live run is stopped and begun
      again at Tick `0` at its period.
- [x] `InputEvent::Rewind`, and the console translates Command Shift R into it
      on native, once per press.
- [x] Engine tests: a resumed run continues the Tick; a Clock resumes on its
      step; a Rewind zeroes the Tick and Run Clock; a live Rewind silences,
      discards the owed Note Off and begins at Tick `0`; the mailbox keeps a
      Rewind across a stop.
- [x] `Orcvs` test: Space pauses and resumes at the next Tick, and a Rewind
      returns to Tick `0`.
- [x] Console tests: the chord translation, and a kittest that Space pauses
      and resumes and Command Shift R rewinds.
- [x] The tests that assumed a new run begins at Tick `0` now pin resume.
- [x] ADR 0076 is written, ADR 0012's and ADR 0056's Status lines name it, and
      `CONTEXT.md`'s Clock, Delay, Random, Playback Engine, Run Clock and new
      Rewind entries match.
- [x] Gates pass with `PROPTEST_CASES=32`.

## Comments

2026-10-18: Built on `claude/playback-pause`, its own branch and PR, apart from
the tracker examples work it aligns with Orca.
