# 06 — Close the remaining PR #213 review findings

Status: resolved
Blocked by: 01, 02, 03, 04

**What to build:**

The review findings on PR #213 that the Bang Function does not remove. Ticket 01 removes the provenance findings (anchor mismatch, one-Tick expiry, test helpers planning without the display, the five-argument clump, duplicate roots in the typed-Bang loop); confirm each is gone and close the rest.

## Acceptance criteria

- [x] A test pins that a Push whose answer is a Bang writes `**` over its complete destination and activates the roots aligned with it, as the Copy, Read, Track and Write tests do.
- [x] The Bang-root alignment doc no longer claims every caller skips intrinsically active roots.
- [x] The vertical-Copy test comment no longer says a typed `**` is not a Bang a Copy can carry.
- [x] Whether a Function answers only Bang is a column of the Function declaration table, not a separate list, and the whole-list test reads it from there.
- [x] A Function reference example can again state that it writes no Cell.
- [x] Each provenance finding from the review is confirmed absent after ticket 01, with the test or code fact that shows it.

- [x] Tickets 01–04 pass together before the feature is marked complete; ADR 0072 lists every Orca departure in the map.
- [x] `lang/bang-function-prototype.html` is marked non-authoritative or removed.

## Comments

2026-10-11 — Review findings closed:

- Push: `a_bang_pushed_overwrites_its_whole_pair_and_activates_the_roots_aligned_with_it` (`orcvs/src/source/tick/push.rs`).
- `Relationships::bang_roots`'s doc now says scheduling orders nothing after an intrinsically active root and a Bang reaching one changes nothing, rather than that every caller skips it; execution still marks such a root activated.
- The vertical-Copy comment in `nested_copy/vertical_and_map.rs` says the typed `**` is a Bang Function whose Cells the Copy reads after its Turn.
- The Function table's Bang column is now `Never`, `May` or `Only` (`lang/src/atom.rs`); `can_emit_bang` and `answers_only_bang` both read it, and `exactly_the_bang_ports_answer_only_bang` sweeps `Function::ALL` through it.
- `ExpectedResult` can again state that an example writes no Cell, and the absolute Copy and absolute Write examples state it for the pair south of their anchors.

Provenance findings, confirmed absent after ticket 01:

- Anchor mismatch: there is no record to disagree with a reparse; `a_lone_star_typed_beside_a_written_bang_fires_nothing_and_survives`.
- One-Tick expiry: nothing expires; `a_bang_in_an_operand_fires_exactly_once_when_freed` frees the `**` after zero, one and three quiet Ticks.
- Test helpers planning without the display: `plan(grid, cells, map, tick)` takes no Bang argument, and every helper calls it as `Source::plan_tick` does.
- The five-argument clump: `plan` is back to four arguments, and `execute`'s signature is `main`'s.
- Duplicate roots in the typed-Bang loop: the loop is gone with the start-of-Tick fire; a Bang Function activates through `bang_roots` like every producer.

