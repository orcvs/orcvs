# 01 — Give a late Tick one policy

**What to build:** One rule for the deadline a Playback run could not reach in time,
recorded as an ADR and held by both targets.

**Blocked by:** None.

**Status:** resolved

- [x] The rule is recorded as an ADR and its vocabulary reaches `CONTEXT.md`.
- [x] The native clock and the browser clock hold the same rule.
- [x] A test fails if either target changes its answer, and a test fails if the two
      answers diverge from each other.
- [x] The test that pinned the previous native behaviour is replaced rather than
      deleted, and its replacement distinguishes all three candidate rules.

## Comments

The divergence was found while scoping effort 02, not by a gate.

`3abd550` added `set_missed_tick_behavior(Skip)` to the app-level ticker, in the same
commit that introduced the scheduled/observed pair and overrun detection. `602399d`,
four hours later, moved the loop into the Playback Engine and rewrote it with `Burst`;
its subject is "Make Playback Engine own clock scheduling" and it does not mention a
policy. `28e22e5`, two days later, ported the loop to wasm32 — `tokio::time::Interval`
does not run there — and hand-wrote deadline arithmetic that comes out as tokio's
`Delay`. Its subject is "fix: support browser playback and unify verification".

`602399d` also added `playback_clock_reports_each_overrun_and_resumes_without_wall_clock_sleep`,
whose two assertions of `2` are `Burst` and nothing else. Confirmed by experiment before
any of this was written: switching that one word to `Skip` failed the test at
`assert_eq!(adapter.command_lists().len(), 2)` with `left: 1, right: 2`. A regression
that arrives with its own passing test is invisible to CI indefinitely.

ADR 0002 was not violated. It never asked the question, which is the finding this
effort acts on.
