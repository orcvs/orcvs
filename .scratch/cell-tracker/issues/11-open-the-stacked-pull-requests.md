# 11 — Open the stacked pull requests

Status: wontfix
Blocked by: 09, 10, 12

**What to do:**

Push the three stacked branches to `origin` (`orcvs/orcvs`) and open one pull request each, each based on the branch below it:

1. `cell-tracker/sequence-retirement-and-return` → `main` (tickets 01–02, plus review fixes `8326627a`)
2. `cell-tracker/blank-results` → `cell-tracker/sequence-retirement-and-return` (ticket 03, 08, 09)
3. `cell-tracker/track` → `cell-tracker/blank-results` (ticket 04, 10, 12)

## Acceptance criteria

- [ ] PR 1 description states that the Theme keys `source.sequence`, `source.sequence.background` and `source.atom.background` are removed; Theme documents reject unknown fields, so a custom Theme naming one fails to load.
- [ ] PR 1 description notes that the test `live_a_rejected_tick_records_no_turn_for_the_computations_it_never_reached` was removed because no Source without Sequences reaches that rejection; `a_late_spatial_write_rejects_the_tick_even_when_the_earlier_turn_failed` still covers the unreached-Turn guarantee.
- [ ] PR 3 description names the fifth Function Replacement fact, `ReplacementChange::List`, which ticket 04 did not list, and why it exists (a replaced Track would have no Items and stop without a diagnostic).
- [ ] PR 3 description notes that spec decision 11 (a rest does not cancel running Timed Play notes) is deferred to ticket 06.
- [ ] Ticket files 01–04, 08–10, 12 are set to `Status: resolved` with their checkboxes ticked once merged, and the roadmap gates pass: `node --test scripts/tests/roadmap.test.ts` and `node scripts/roadmap.ts > /dev/null`.

## Comments

Stack opened 2026-10-05: #199 (01–02) → #200 (03, 08) → #201 (04, 10) → #202 (09) → #203 (12). Still open: the Theme-key note on #199, and resolving ticket statuses after merge.

Superseded by PR #205, which replaced the stack. #199 merged. #200 merged after its branch was rewritten: it carries ticket 03's pending behaviour (ADR 0066), not ticket 08's commits, which never reached `main`. #201, #202 and #203 were closed.

The merged #199 description lacks two notes, recorded here instead:

- Commit `5e8a307a` removed the Theme keys `source.sequence`, `source.sequence.background` and `source.atom.background`. Theme documents reject unknown fields (`console/src/theme_document.rs`), so a custom Theme naming one fails to load.
- The same commit removed `live_a_rejected_tick_records_no_turn_for_the_computations_it_never_reached`. Its guarantee belonged to a Tick that is rejected, and ADR 0065 rejects no Tick, so it has no direct successor.

The test the criterion names as covering it is now `a_late_spatial_write_is_refused_and_the_tick_continues_even_when_the_earlier_turn_failed` (`orcvs/src/source/tick.rs`). That test asserts that every computation takes a Turn, so it covers no unreached Turn. The nearest surviving guarantee, that a computation stopped by a cycle discovered during the Tick takes no Turn (ADR 0068), is covered by `a_cycle_discovered_by_a_nested_track_stops_its_sibling` in `orcvs/src/source/tick/track.rs`.

The criteria stand as written and are not met, because the work they describe did not merge as planned. Tickets 01–04 are `resolved` with their boxes ticked. Tickets 08, 09, 10 and 12 are `wontfix`: their commits never reached `main` (see each ticket's Comments). #201 closed, so PR 3 has no description: `ReplacementChange::List` is absent from `main` because PR #205 replaced the List-claim Track, and spec decision 11 is carried by ticket 06.
