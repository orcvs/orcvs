# 02 — Give the note schedule its own module

**What to build:** Lift `OwnedNotes` and the types it owns — `Voice`, `Sounding`, `Claim`, `Expiry`, and `note_off` — out of `playback.rs` into `orcvs/src/playback/schedule.rs`, `pub(crate)`, and move the tests that are about it onto it.

It is already a deep module: four methods over two maps, entirely pure — no clock, no lock, no adapter. What it lacks is a name and a home, so the tests that are about it reach it through the engine instead. One test already skips that: `a_chord_of_timed_plays_stops_each_element_at_its_own_length` constructs `PlayCommand`s, calls `deliver` four times, and is thirty lines with no Source at all. The other ownership tests build a `Grid`, write an Orcvs Expression at a magic Cell index, write `.=0101` as a Bang generator and `.=0102` to retire it, and reach the schedule through `activate_for_test` and `clock_tick`. To assert that a Monophonic Play stops whatever note its channel was sounding, a maintainer currently needs to know Equality's Bang alignment and the Grid's linear indexing.

**This is what makes `04` affordable, and that is why it is here rather than in the architecture backlog.** Of 61 tests in `playback.rs`, 34 drive only the `#[cfg(test)]` doors and 9 mix them with the handle. The great majority of those 34 are ownership, expiry and delivery-order tests — this module's subject. Moving them off the engine leaves `04` facing the tests that are genuinely about lifecycle.

It also honours `.scratch/playback-clock/spec.md`'s reason for keeping the door rather than overriding it. That spec declined to reshape the 58-use door because those tests "exercise Tick execution — ownership, expiry, delivery order — and driving them through a clock would make them slower without making them truer". Correct. They are not driven through a clock here; they stop going through the engine at all.

Cases currently unreachable become reachable and should be added: two Mono claims on one channel expiring at the same Tick, a Timed claim whose `Tick::after` saturates, and `expired_at` draining a range wider than one Tick — the behaviour its own doc comment describes and no test exercises.

**Blocked by:** 01

**Status:** resolved

- [x] `OwnedNotes` and its types live in `orcvs/src/playback/schedule.rs`, `pub(crate)`.
- [x] The ownership, expiry and delivery-order tests address it directly, in the numbers ADR 0016 uses, with no Grid and no Source.
- [x] The engine's remaining tests assert only what the engine adds: that a Tick's delivery is one submission, that a refusal leaves the copy unadopted, and that lifecycle clears the schedule.
- [x] The three unreachable cases above have tests.
- [x] No behaviour changes. ADR 0016's ownership rules are untouched.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package <crate> --all-targets --locked -- -D warnings` and `PROPTEST_CASES=32 cargo nextest run --package <crate> --locked` for `orcvs` and `console`.

## Comments

`OwnedNotes` and the types it owns — `Voice`, `Sounding`, `Claim`, `Expiry` and `note_off` — now
live in `orcvs/src/playback/schedule.rs`. The Rust 2018 layout was kept: `playback.rs` beside a
`playback/` directory, which is the arrangement `orcvs/src/source/tick.rs` already uses beside
`source/tick/execution.rs`, so no `playback/mod.rs` was introduced.

`OwnedNotes` is `pub(crate)`, as are the two methods the engine calls (`deliver`, `clear`) and a
`#[cfg(test)]` `holds_note_ownership`, which the engine's own test-only query now delegates to
instead of reaching into the two maps. `Voice`, `Sounding`, `Claim`, `Expiry` and `note_off` stay
private to the module: nothing outside it names them, and widening them would have been surface
the engine does not use.

Twenty-two tests moved. Twenty-one of them reached the schedule through a `Grid`, a Source
Expression at a magic Cell index, a `.=0101` Bang generator and the `activate_for_test` /
`clock_tick` doors; they now construct `PlayCommand`s and call `deliver` per absolute Tick, in the
numbers ADR 0016 uses, with no `Grid` and no `Source` anywhere. The twenty-second,
`a_chord_of_timed_plays_stops_each_element_at_its_own_length`, already had that shape and moved
verbatim. One was renamed, because "reaches the adapter" stopped being true of it:
`control_change_and_pitch_bend_reach_the_adapter_unresolved_and_in_tick_plan_order` is now
`control_change_and_pitch_bend_are_delivered_unresolved_and_in_tick_plan_order`. The Source side
of that test — that `!c` and `!b` reach a Tick Plan as those Play Commands at all — is already
covered by `orcvs/src/source/model.rs`, so nothing was lost with the Grid.

Four tests were added, all of them cases the engine could not reach:

- `two_mono_claims_due_at_one_tick_stop_only_the_voice_still_standing` — `03` from Tick 0 and `02`
  from Tick 1 both fall due at Tick 3, so one voice has two expiries in one drain. One stop is
  delivered, for the note the surviving claim recorded.
- `a_lifetime_that_saturates_the_last_tick_leaves_a_stop_no_tick_can_reach` — a Timed claim whose
  `Tick::after` saturates. `expired_at` drains everything strictly before `tick.next()`, and
  `Tick::next` saturates at the same Tick, so the stop is beyond every Tick the counter can reach.
  That is what `expired_at`'s doc comment says and nothing asserted.
- `a_delivery_drains_every_stop_due_before_it_and_not_only_this_tick_s` — the range wider than one
  Tick the same doc comment describes: two stops due at Ticks 2 and 4, both delivered by a single
  delivery at Tick 9, in due-Tick order.
- `clearing_forgets_every_claim_and_every_scheduled_stop` — `clear` had no direct test; the engine
  asserted it only through `holds_note_ownership` after a lifecycle action.

Each of the four was driven red by a targeted mutation of the code it covers and restored
afterwards (removing the voice retirement in `expired_at`, and again together with the claim
comparison, gave the doubled stop; draining `<= tick` inclusively rather than via `tick.next()`
made the saturated stop reachable; draining only the exact Tick emptied the wide drain; dropping
`expiries.clear()` left the schedule holding). The twenty-two moved tests passed before the move
and pass after it, which is the discipline a pure move owes instead.

No behaviour changed. The only non-test edits to `playback.rs` are the module declaration, the
import list, one doc link that had to become fully qualified once `PlayCommand` left the file, and
`holds_note_ownership` delegating.

Ticket `04` inherits a smaller surface than the spec predicted: `playback.rs` now holds 39 tests,
15 of which touch the `#[cfg(test)]` doors (13 door-only, 2 mixed with the handle), down from 43
(34 door-only, 9 mixed). `schedule.rs` holds 26.
