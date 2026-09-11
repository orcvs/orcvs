# 02 — Add Source Bang activation and expiry

**What to verify:** Close the original Bang-lifecycle issue against `CONTEXT.md` and ADR 0032. Implementation is tracked in `tick-execution-order/02`; do not implement the old cleanup-turn design independently.

**Blocked by:** None — tick-execution-order/03 is resolved.

**Status:** resolved

**Tags:** release/v1

- [x] Current-Tick Bang output activates aligned cardinal Expression roots with settled operands.
- [x] Horizontal root anchors are two columns away; vertical anchors are one row away, in every
      direction Source can author. The west anchor's positive case is unauthorable until a root
      declares no operand and belongs to `spatial-tick-planning/03`; its negative case is covered.
- [x] Dependencies place a Bang producer before its consumer regardless of Position; each root executes at most once.
- [x] Prior Bang display is discarded without activation; manually entered `**` is a no-op.
- [x] A current result remains visible until the next Tick without becoming another activation.
- [x] Direct activation does not overwrite the target Function.
- [x] Partial or non-root contact does not activate.
- [x] Parsing owns contextual Bang validity; rejected typed operands neither activate nor receive display cleanup.

## Comments

2026-09-06: The accepted dependency-ordering design supersedes this ticket's original snapshot-activation and cleanup-turn requirements. See ADR 0032 and `tick-execution-order/01`–`03`. This issue remains an integration acceptance record, not a second implementation workstream.

2026-09-06: ADR 0032's fixed-Portal scheduler now provides current-Tick dependency ordering, transient display, manual no-op, parser-owned validity, and once-only execution. This ticket remains open: current Source grammar cannot author a horizontally adjacent standalone Bang without merging it into the target Expression, and the Directional/Self-Banging Function surface that makes horizontal contact useful is still follow-up work.

2026-09-09: Verified seven of the eight items against `main`, per item, by reading the code that
decides the behaviour and then confirming a test actually drives it. Where a claim held but nothing
drove it, the direction was established by mutation rather than assumed: each guard was broken in
turn and the suite re-run, so "covered" below means a test failed when the behaviour was removed.

- **Cardinal activation with settled operands.** `bang_roots` names the four anchors
  (`orcvs/src/source/tick.rs:217`) and execution sets `activated[owner]` from a current-Tick Bang
  result only (`orcvs/src/source/tick.rs:456`). Covered by
  `a_bang_activates_its_aligned_neighbours_and_no_further_root`
  (`orcvs/src/source/tick.rs:1589`) and, for the settled-operand half,
  `fixed_upward_portals_schedule_note_and_bang_before_midi`
  (`orcvs/src/source/tick.rs:1935`), which sounds `C4` rather than the typed `D4` because the Note
  supplier settles before the activated root runs. Removing the north or south anchor fails the
  first; removing the Bang dependency edge (`orcvs/src/source/tick.rs:682`) fails both.
- **Anchor distances.** Left unticked. See the separate note below.
- **Dependency order and once-only execution.** Bang edges are established before execution
  (`orcvs/src/source/tick.rs:682`) and the topological pass visits each node once
  (`orcvs/src/source/tick.rs:706`). `fixed_upward_portals_schedule_note_and_bang_before_midi`
  (`orcvs/src/source/tick.rs:1935`) puts both producers *below* the MIDI root, so Position is
  contradicted rather than merely absent. `two_current_bang_results_still_execute_midi_once`
  (`orcvs/src/source/tick.rs:1968`) pins the single execution, and
  `the_interpreter_is_handed_the_shared_tick_and_each_roots_own_anchor`
  (`orcvs/src/source/tick.rs:1897`) counts the Evaluator calls. Issue 01 handed this ticket the
  row-major tie-break guard it could not pin end to end; ADR 0032's ready set orders by
  `CellIndex` (`orcvs/src/source/tick.rs:699`) and reversing that pop order now fails 17 tests, so
  the guard 01 called "not load-bearing today" is load-bearing and pinned.
- **Transient display and the manual no-op.** The pre-execution clear reads parsed Bangs only
  (`orcvs/src/source/tick.rs:299`, over `LanguageMap::bangs` at
  `orcvs/src/source/language_map.rs:341`). Removing the clear fails
  `a_generated_pulse_does_not_replay_on_the_next_tick` (`orcvs/src/source/model.rs:1267`). The
  manual case is `manually_entered_bang_is_display_only_and_never_activates_midi`
  (`orcvs/src/source/model.rs:1287`) — a hand-typed `**` one row from a Raw Play sounds nothing
  and is cleared.
- **Visible until the next Tick, without a second activation.**
  `a_generated_pulse_does_not_replay_on_the_next_tick` (`orcvs/src/source/model.rs:1267`) asserts
  the committed `**` in row 1 after the first Tick, then stops the producer and asserts silence
  and a cleared row on the second.
- **Activation does not overwrite the target.**
  `a_result_beside_a_root_preserves_the_root_and_activates_it_each_tick`
  (`orcvs/src/source/model.rs:1471`) holds this where it can actually be contested — same row,
  Cell-adjacent — asserting `**!>007FC4` unchanged across three Ticks while the root performs on
  each. Deleting the east anchor fails it. The delivery-*onto*-a-root form `CONTEXT.md` gives Jump
  ("activates an Expression root without overwriting it") is not this mechanism and belongs to
  issue 04.
- **Partial or non-root contact.** Partial contact was already covered: the whole-pair operand test
  in `bang_roots` (`orcvs/src/source/tick.rs:218`) is driven by
  `a_bang_half_inside_an_operand_does_not_activate_an_aligned_terminal`
  (`orcvs/src/source/tick.rs:1675`),
  `a_bang_touching_a_nested_function_operand_does_not_activate_a_neighbour`
  (`orcvs/src/source/tick.rs:1696`) and
  `live_bang_in_half_typed_terminal_claim_diagnoses_without_activation`
  (`orcvs/src/source/tick.rs:1069`). **Non-root contact was prose only.** `root_at`
  (`orcvs/src/source/tick.rs:166`) requires the anchor to be both exact and parentless, and no test
  drove the parentless half — dropping it left all 299 tests green. Added
  `a_bang_cardinally_aligned_with_a_nested_function_anchor_activates_nothing`
  (`orcvs/src/source/tick.rs:1713`): the Bang gets a clear row of its own so operand contact cannot
  answer it, and its southern anchor lands on a nested `.+`. It is the only test in the suite that
  fails when `root_at` is made to route non-root contact up to the owning root, which is the
  plausible regression this item forbids.
- **Parser-owned validity.** `LanguageMap::bangs` admits Bang Atoms from complete standalone
  Expressions only (`orcvs/src/source/language_map.rs:341`), so a rejected operand spelling is
  never a clear candidate. Covered end to end by `a_rejected_bang_operand_neither_activates_nor_erases`
  (`orcvs/src/source/model.rs:1315`), which asserts no command, no write, an unchanged Snapshot and
  a surviving syntax diagnostic; and at the parse layer by
  `invalid_operand_bang_spellings_are_not_parsed_bang_values`
  (`orcvs/src/source/language_map.rs:685`) and `a_bang_rejected_in_a_typed_operand_never_activates`
  (`orcvs/src/source/tick.rs:1733`).

**Why the anchor-distance item stays unticked.** The distances themselves are right and tested: one
row vertically, positively north and south, with the root two rows away asserted silent
(`orcvs/src/source/tick.rs:1589`); two columns horizontally, positively through the `column + 2`
lookup (`orcvs/src/source/model.rs:1471`), with one and three columns asserted silent in the table
at `orcvs/src/source/tick.rs:1626`. The `column - 2` lookup (`orcvs/src/source/tick.rs:233`) is a
different matter: deleting it leaves the whole suite green, and it is unreachable rather than merely
untested. Every Function in `define_functions!` (`lang/src/atom.rs:745`) declares at least one
operand, and the Self-Banging spellings `^^ vv << >>` are `Atom::Activation`, not `Function`, so
there is no zero-operand root. A root anchored at column `c-2` therefore always claims column `c`
for its first operand, and a Bang at `c` is operand contact — confirmed by probe, which produced
`!>**7FC4` and no activation. The `(6, 2)` row of `orcvs/src/source/tick.rs:1626` already asserts
exactly that, so the negative side is covered and only the positive side is unauthorable.

This also sharpens the 2026-09-06 note above, which was too broad: a standalone Bang *west* of its
target is authorable today and works (`**!>007FC4`). Only a Bang *east* of its target merges into
that target's operands.

**What blocks the last item, and the decision a human owes it.** The `column - 2` direction becomes
reachable when `<<` and `>>` are scheduled root Functions with no operands — that is issue 03's
work, and 03 carries the checkboxes for it ("Complete root contact can activate; partial Language
Unit contact diagnoses and activates nothing"; "Tick-by-Tick Source Grid tests cover all four
directions and row edges"). No new issue was filed, because 03 and 04 already own both deferred
surfaces. But issues 03, 04 and 05 are each **Blocked by: 02**, so holding 02 open for a surface 03
delivers is circular, and the release cannot move through it. Either resolve 02 on ADR 0032's
delivered scope and let 03 carry the west-direction coverage, or restate 02's second item as the
distance claim it verified. That choice is a scope decision rather than agent work, which is why the
status is now `ready-for-human` rather than resolved.

Verification: `cargo fmt --all -- --check`; `cargo clippy --package orcvs --all-targets --locked --
-D warnings`; `cargo clippy --package shell --all-targets --locked -- -D warnings`;
`cargo nextest run --package orcvs --locked` (300 passed); `cargo nextest run --package shell
--locked` (35 passed), with `PROPTEST_CASES=32`. `mise run check` and `mise run test_persistence`
are deferred to CI. The change is one added test and touches no production path.

2026-09-10: Resolved on ADR 0032's delivered scope. The second item is restated as the claim the
2026-09-09 verification established rather than the one it could not reach. The distances are right
and driven in three directions: one row north and south with the root two rows away asserted silent
(`a_bang_activates_its_aligned_neighbours_and_no_further_root`), and two columns east through the
`column + 2` lookup, with one and three columns asserted silent in
`fixed_bang_destinations_respect_alignment_and_operand_contact`.

The `column - 2` lookup is unreachable rather than untested. Every Function in `define_functions!`
declares at least one operand, so a root anchored two columns west always claims the Bang's own
destination for that operand, making a Bang there operand contact — which the `(6, 2)` row of that
same table asserts. There is no zero-operand root to author the positive case against: `^^ vv << >>`
exist today as `Atom::Activation` (`lang/src/atom.rs:206`), not as scheduled Function roots.

Three consequences, all landed with this resolution:

- `CONTEXT.md`'s Bang entry now states the anchor geometry in the words its Jump entry already used,
  so the rule has a source of truth outside `bang_roots` and this ticket's prose.
- The `column - 2` arm is kept, with a doc comment on `bang_roots` naming why it is unreachable, why
  it is not deleted, and which ticket makes it live.
- Issue 03 gains an explicit item for the west-anchor activation. Its existing items describe a
  Self-Banging Function moving *into* a root; this is the converse — a Bang result finding a
  zero-operand root at its west anchor — and no item covered it, so handing the coverage over
  without one would have lost it.

Resolving this clears `Blocked by: 02` on issues 03, 04 and 05, and the release critical path with
them.
