# 02 — Concentrate the clock loop

**What to build:** One clock loop per target instead of four, with the deadline rule as
a single shared function rather than two implementations that must agree.

**Blocked by:** `playback-clock/01` — the behaviour change lands first, so this one is a
refactor with no behaviour to argue about.

**Status:** ready-for-agent

- [ ] `start` and `retune` no longer each carry a clock loop; they differ in the first
      deadline they supply and in whether they begin a run.
- [x] The deadline rule is computed in one place for both targets. The native clock no
      longer delegates it to `tokio::time::Interval`'s missed-tick machinery, so the
      only remaining per-target difference is how to wait until an instant.
      Landed with `playback-clock/01`: holding one rule on both targets forced it.
- [ ] Cancellation, the `ClockRunGuard`, the `Weak` upgrade, and Tick delivery are
      written once.
- [ ] `start`'s immediate first Tick and `retune`'s anchoring to the last executed Tick
      both survive, with tests naming them.
- [ ] No behaviour changes. Effort 01's tests pass unaltered.

## Comments

Two architecture reviews reached this from opposite directions. The 2026-09-09 review
("Give the Playback clock a real seam") wanted a `Clock` trait with tokio, browser and
manual adapters. The 2026-09-10 review ("Concentrate Playback clock execution") wanted
no trait at all, and treated the two targets' timing as behaviour to preserve.

Effort 01 settles that: there was no target-specific behaviour to preserve, which
removes the reason the second review rated this second-wave work. What the reviews both
missed is that the thing worth sharing is the deadline rule, not the loop scaffolding —
the scaffolding is only how the rule came to be written twice.

The manual test adapter the first review proposed is out of scope; see the spec.

One box arrived already ticked. Effort 01 could not make both targets hold one rule
without abandoning `tokio::time::Interval`, so the shared `next_scheduled_at` and the
`sleep_until` loops landed there. What remains here is the scaffolding: four loop
bodies, four copies of cancellation, the `ClockRunGuard`, the `Weak` upgrade and Tick
delivery.

Concentrating those loops is also what would let one test cover both targets. Today
nothing in the workspace compiles the browser loops, so the rule is shared by
construction — both call `next_scheduled_at` — rather than by a test that would fail if
they stopped. See the note on effort 01.

Three corrections from the review of effort 01, which found two deadline defects and
fixed both by hand where this effort would have made one of them unwritable.

The loop shapes are not a decision to preserve. Browser `start` is execute-then-sleep
and the other three loops are sleep-then-execute, and nothing in ADR 0037, `CONTEXT.md`
or any comment says why. `28e22e5` hand-ported the native `time::interval_at` loop,
whose first `tick()` returns immediately, and reproduced that by moving the sleep to the
bottom; `bcc95c7` then wrote browser `retune` in the native shape, because its first
deadline is a period away rather than zero. The shape follows mechanically from whether
the first deadline is `ZERO`, so the browser already holds both. It is the same
unrecorded transcription artefact ADR 0037 was written about. What is real underneath it
is one platform fact: `tokio::time::sleep_until` on an elapsed deadline is `Ready` on
first poll, while `TimeoutFuture::new(0)` costs a `setTimeout` hop of one to four
milliseconds — enough for `is_overrun` to decline the first Tick of a browser run at the
one-millisecond end of `Bpm`. State the wait as "a deadline already reached costs no
wait" and both targets hold the sleep-then-execute shape with no semantic change. That
narrows the always-schedule-a-timer rule `0ab1a4c` introduced, so
`zero_wasm_delay_still_schedules_a_browser_timer` has to be replaced by one pinning
sub-millisecond rounding. The criterion above that asks for effort 01's tests to pass
unaltered has that one exception.

The seam is two free functions and a spawner value, not a trait. A `sleep_until` per
target, mirroring how `ClockInstant` is already one alias per target, plus a spawner
obtained before the lock is taken — native `retune` proves the runtime present before it
cancels the previous clock and bumps the generation, and a spawn that failed after that
would leave a playing engine with no clock. `Send` is not a constraint: both loops
already live in `impl<A: OutputAdapter + Send + 'static>`, and a shared `async fn` takes
its auto-traits per instantiation, so the native one satisfies `tokio::spawn` and the
browser one does not have to.

The note above about nothing compiling the browser loops is stale, and it understates
the real gap. `check_wasm` runs `cargo clippy --workspace --all-targets --target
wasm32-unknown-unknown` on every pull request, so both loops type-check, and
`shell/tests/wasm.rs` drives `PlaybackEngine::start` in headless Firefox in the merge
tier. What is missing is narrower and worse: no test on any target asserts a browser
deadline, and browser `retune` — where effort 01's second defect lived — is executed by
nothing at all, because it is `pub(crate)` and `wasm.rs` never calls it. This effort
should add a deadline assertion to `shell/tests/wasm.rs` and a path that drives `retune`
there. Concentrating the loops closes the divergence; it does not close the sleep
primitive itself, where `wasm_timeout_millis` rounding, `setTimeout` clamping and
background-tab throttling stay browser-only and native-untestable.
