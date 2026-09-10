# 05 — Migrate the ordering and dependency destination tests

**What to build:** The first migration batch. Move the tests whose subject is dependency ordering —
which producer takes its Turn before which consumer, what a cycle rejects, what an inactive owner
suppresses, and what a failed producer leaves standing for its spatial consumers — onto the
constructed schedule from `04`. Their assertions do not change.

Batch boundaries are sized to keep the tree green between tickets rather than drawn from any rule;
an implementer may move the line between `05`, `06` and `07` as long as each lands green on its own.

**Blocked by:** 04 — Build a destination-carrying schedule a test can construct.

**Status:** resolved

- [x] Every test in this batch drives execution through the constructed schedule, except the
      sequence fixtures named below, which no test can move one at a time.
- [x] Every behaviour the batch asserted before it moved is still asserted after.
- [x] The crate builds and its tests pass; the configured route still exists for the batches that
      have not moved.

## Comments

Split out of the architecture review of the Sequence branch, 2026-09-10.

Carry this batch's share of `10` rather than repeating it: these tests change route here, and the
state record `10` introduces is what they read once they have. Changing them twice is the cost of
sequencing the two apart.

2026-09-10: Resolved by the commit this line lands in. Nine tests moved, all in
`source::tick::test`, and no assertion changed in any of them:

- `live_cross_boundary_chain_uses_lower_producers` and `partial_writers_settle_before_consumption`
  — which producer takes its Turn before which consumer.
- `live_cycles_reject_independent_effects_and_self_dependency` and
  `live_inactive_nested_portal_cannot_create_a_cycle` — what a cycle rejects, and what an inactive
  owner keeps out of one.
- `live_inactive_ownership_and_terminal_portal_configuration_are_independent` — what an inactive
  owner suppresses.
- `live_failed_suppliers_preserve_spatial_data_but_not_nested_answers`,
  `live_child_write_survives_parent_failure_and_rejected_portal_keeps_typed_answer` and
  `an_activated_consumer_uses_surviving_cells_after_supplier_failure` — what a failed producer
  leaves standing for its spatial consumers.
- `a_late_spatial_write_rejects_the_tick_even_when_the_earlier_turn_failed` — the same subject at
  the execution seam.

Eight of the nine were a change of helper and nothing else, as `04` promised. The ninth built its
own `Configuration` to reach `plan_configured` and `schedule` directly, so it now builds the same
destinations through `carried_destinations` and reaches `plan_carrying` and `schedule_carrying`.
That replaced its insertion loop with the one call the helper already is, which is the only line of
any migrated test that is not a substituted identifier.

Where the line was drawn, for whoever takes `06` and `07`. Two tests straddle and were taken here
because their first clause is this batch's subject, not `06`'s:
`live_child_write_survives_parent_failure_and_rejected_portal_keeps_typed_answer` also asserts a
row-edge refusal, and `partial_writers_settle_before_consumption` also has two writers landing in
one operand. Three straddle the other way and were left: the test that decodes a pending note only
after every writer has settled, the one that orders competing writers by Position, and
`live_unchanged_nested_syntax_errors_do_not_repeat_as_tick_failures`, which repairs a leaf before
its reserved Turn in its second half but is a test about diagnostics that do not repeat.

What remains on the configured route, in two shapes rather than one. Thirty-seven
`configured_source` call sites are still there — the write composition tests, the Bang activation
and Function replacement tests, and the tests that state no destination at all and are on the
helper only because it is the one that seeds a Source. Two of the thirty-seven belong to
`a_carried_destination_schedules_the_tick_a_configured_one_would_have` and
`a_refused_portal_is_diagnosed_before_the_row_edge_layout_it_shares_a_tick_with`, which drive both
routes on purpose and stay until `08` deletes the configured one.

Separately, `plan_with_destinations` has nine call sites across seven tests — the Bang activation
and Pulse tests, the row-edge destination test, and
`competing_writers_publish_but_dependency_cycles_abort_before_output`, which holds two of them.
That route is retired by whichever batch takes the last of the seven, not by any one of them, so a
ticket that migrates only some of its callers should expect the route to remain. The competing
writers test was left whole because its two halves belong to two different batches.

`execution::stated::plan_with_answers` still takes a `Configuration`, as `04` left it, so the
`stated_source`, `replaced_source` and `sequence_source` fixtures are all still configured. Some of
what they cover is this batch's subject — a Sequence reservation ordering what its write reaches,
and a reservation covering its own producer. They did not move, because they cannot move one at a
time: their destinations are stated inside the helper, so swapping its parameter migrates every
caller in one step, across all three batches at once. That is one route swap and it belongs to
whichever ticket takes the sequence tests, or to `08`.

`10`'s share for this batch is not incidental and was not taken. Three migrated tests read the
`observed` thread-local — the two cycle and inactive-owner tests assert that nothing was
interpreted, and the late-spatial-write test asserts how many Turns reached the Interpreter. The
carried route gives them no way to stop: `plan_carrying` and `Source::execute_carrying` return a
`TickPlan`, exactly as their configured twins do, and `04` added no second output. Reading those
counts from execution instead means the whole of `10` — a field on `ComputationState`, `execute`
returning the states beside the plan, planning forwarding them and the Source discarding them —
which is shipped code and `10`'s to write. So the three change twice, which is the cost `10`'s own
comment names, and the route change is the half that could be done here.
