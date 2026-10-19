# 03: Light the Cells a Tick wrote

**What to build:** Every delivery at an Output Portal site is recorded, admitted
or refused, and admitted writes paint as Output Portal, so a selected write
(Write, Push, `=$`) lights where it landed this Tick. See `../spec.md`
sections 1-3 and 6.

**Blocked by:** 02

**Status:** resolved

- [x] A write is recorded for each site delivered to, Declared or Selected,
      with an Output Span when admitted or a Write Refusal that covers every
      refusal: geometric (with the attempted Position where one exists),
      Function Replacement, and ordering.
- [x] Output Spans can be made only from an admitted write.
- [x] A destination that resolved for a Turn whose interpretation failed is
      not recorded.
- [x] Admitted writes paint as Output Portal; refused writes do not paint.
- [x] Tests: each refusal keeps its reason and paints nothing; a Cell both read
      and written paints as Output Portal; an empty read Cell shows the Input
      Portal background; order and owners hold when the Tick rewrites the Source
      its owners stood in.

## Comments

2026-10-19 — `PortalEvent::Write { origin, outcome }` is recorded in
`deliver_output` for every site it is asked to deliver to: `Ok(OutputSpan)`,
minted only by `OutputSpan::of(&SpanWrite)`, or `Err(WriteRefusal)`, one of
`Portal { error, at }` (`at` is `None` for a site with no Position in the Grid
and the attempted Position when admission at it refused), `Replacement(change)`
and `OrderingDefect`. The origin is `Selected` for `Delivery::Selected` and
`Declared` for `Reserved` and `Joined`. The admitted entry is pushed after the
whole-write refusal and before the Contact loop, so its owner is the Function
the Turn ran even when the write replaces one. A Turn whose interpretation
fails never reaches `deliver_output`, so its resolved destination is absent
without a guard of its own. `RenderFrame::derive` folds admitted writes into
`PortalRole::Output` in the same in-place pass as reads, beside the static
highlight, so a Function that wrote nothing this Tick still shows its own
Output Portal; the allocation test now carries a selected write and still
allocates no more than a bare revision. Two tests could not go red first:
`a_destination_the_turn_resolved_before_its_interpretation_failed_is_not_recorded`
holds an absence that held before any write was recorded, and
`an_empty_cell_a_tick_read_shows_the_input_portal_background` holds paint
ticket 02 already made.
