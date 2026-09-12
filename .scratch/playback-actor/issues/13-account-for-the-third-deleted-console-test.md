# 13 — Account for the third deleted console test

**What to build:** A record of `console::midi::playback_start_errors_are_exposed_as_status`, and a
decision about whether what it covered still needs covering.

It existed at `92216af:console/src/midi.rs:178` and is absent at `ffa2dcc`. Neither ticket's Comments
name it — ticket `04` says "Two tests were dropped rather than rewritten" and accounts for exactly
two, `a_tick_the_engine_declines_consumes_no_absolute_tick` and
`retuning_keeps_the_absolute_tick_of_the_run_it_retunes`. This is a third.

What it covered is partly covered generically: `backend_errors_are_exposed_as_status`
(`HEAD:console/src/midi.rs:235`) asserts that an error reaching the diagnostics stream reaches the
status line. What nothing covers specifically is `StartFailure` — the diagnostic a refused `start`
produces — making that trip. `console/src/diagnostics.rs:16` maps it to a message alongside
`ClockFailure` and `RetuneFailure`, so the generic rule does carry it; whether that is enough is the
question.

The deletion is plausibly correct — `Orcvs::new` became fallible, and the test built one
infallibly — but a test that disappears without appearing in either ticket's account is the case the
Comments sections exist to prevent.

**Status:** resolved

- [ ] Ticket `04`'s Comments account for all three deleted tests, or this ticket's answer does.
- [ ] A decision is recorded on whether `StartFailure` reaching the status line needs its own
      coverage beyond `backend_errors_are_exposed_as_status`.
- [ ] If it does, the test is restored in a shape that works with a fallible `Orcvs::new`.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package console --all-targets --locked -- -D
warnings` and `PROPTEST_CASES=32 cargo nextest run --package console --locked`.

## Comments

Filed from the `playback-actor` review ledger as **CR-14**, minor.

Minor because the rule is generically covered and the deletion has an obvious innocent explanation.
Filed because "two tests were dropped" is a claim in a record, and it is off by one.

### Resolved

Accounted for, and the decision is that the coverage was worth restoring in a
different shape.

`playback_start_errors_are_exposed_as_status` cannot come back as it was: it
built an `Orcvs` outside a runtime so that Space produced `RuntimeUnavailable`,
and ADR 0040 moved that failure to construction and made `Orcvs::new` fallible.
The deletion was correct. Nor is there a console gesture left that produces a
`StartFailure` at all — `set_bpm` holds a `Bpm` that cannot be zero, which is
`08`'s subject.

`console::midi::a_refused_start_reaches_the_status_line` states the half that
remains reachable: a `StartFailure` handed to `observe_diagnostics` reaches the
status line. Asserted over `observe_diagnostics` rather than over a staged
refusal, because staging one would mean inventing a caller the console does not
have.
