# 07 — Settle ordered clearing and Halt precedence

Status: resolved
Blocked by: None

## Problem

The retained Bang Function direction leaves two language rules unresolved: Effects from producers that take no effective Turn, and a Halt that depends on the Bang it would lock. Desired examples cannot substitute for ordering rules. This ticket blocks implementation, not merely final documentation.

## Required decisions

- Specify whether syntax-invalid, evaluation-invalid, Halt-locked, inactive, overwritten and cycle-blocked producers clear their Output Portal. Cover nested Bang-only Functions. Record the ordering, validation, refusal and surviving-Bang consequence of each case.
- Specify whether competing Halt activation is determined by actual delivery or potential reachability, how a quiet or suppressed independent source changes the outcome, and what happens when that source depends on the target Bang. Any exception to lock or activation ordering needs an explicit reason; otherwise apply the documented cycle policy.
- Check the spec’s overwrite/emission/movement/read table against these rules. An ordinary overwrite must still suppress yesterday’s Bang, while `*v` activated by its destination Bang must be able to emit after the clear. Record conflicts rather than resolving them with a blanket vacancy predicate.

## Acceptance criteria

- [x] A complete outcome table covers the cases above, including competing writes and failed admission.
- [x] Small executable scheduling examples demonstrate the chosen order or cycle for each case. Clearly distinguish model evidence from production Source Tick tests, which the implementation tickets owe.
- [x] The spec, map and affected ticket criteria agree with the decisions, and explicitly name each Orca departure and reason.
- [x] The remaining implementation tickets are marked ready-for-agent only once their language outcomes are fully specified. Tickets 01–06 remain one completion unit.

## Constraints

Preserve the accepted removal of persistent Bang provenance, all-side activation and surviving-Bang replay. Keep scheduling and ordered Effects behind the Source Tick interface. An unordered cleanup pass is not an acceptable substitute for the clear’s dependencies. Broader changes to accepted desired outcomes require an explicit recorded language decision, not an implementation convenience.

## Decisions

- **D1 — A clear is the producer's ordinary Absence write in its Turn.** A producer that takes no Turn (Halt-locked, inactive, nested in a root that takes no Turn, cycle-blocked) writes nothing. Its last `**` survives and fires once as a Bang Function at its own Turn, as for a removed writer. This matches Orca: a locked `D` does not run and its `*` stands. The halted-clear extension (ticket 02, story 21) is withdrawn; story 21 becomes "a halted producer's last Bang fires once more".
- **D2 — Halt is an ordinary Producer.** Its lock is an Effect of its Turn and holds only if that Turn precedes its target's Turn. Turns are ordered by potential supply (ADR 0032): a Halt waits for every Bang that could reach it, except a Bang from inside its own locked subtree (`lock_covers`, `tick.rs:761-770`). Only a Bang that actually executed activates a Halt.
- **D3 — A Halt above its own activating Bang misses the Bang, not the lock.** For `*!` over `**` over `!>`, `lock_covers` drops the `**`→Halt edge. The Halt takes its Turn inert, the `**` fires (`!>` plays) and clears, and its Bang reaches the taken Halt. That is no cycle. It is diagnosed "Bang reached a root that has taken its Turn" at the Halt. The missed-Bang diagnostic covers every Bang delivered to a taken Turn (D15). Orca departure: Orca's `H` locks the `*` first; Orcvs activates only by delivery.
- **D4 — Clearing outcomes.** ADR 0069 is unchanged; the shared `suppressed` flag never produces a write.

| Producer that banged on Tick T, on T+1 | Writes | Its old `**` |
|---|---|---|
| Valid, answers Absence | admitted clear, ordered before the `**` | suppressed |
| Valid, bangs again | `**` over `**` | old suppressed; new activates in this Turn. Holds for a static writer; a dynamic writer behind another in the writers-first chain can come after its old `**`, which then fires first |
| Halt-locked | nothing | fires once at its Turn, clears |
| Inactive owner, or nested in a root that takes no Turn | nothing | fires once |
| Syntax-invalid, blank operand, evaluation error, Copy with partial input | nothing | fires once |
| Overwritten before its Turn | nothing | fires once unless that overwrite covers it |
| Write refused (out of Grid, row edge) | nothing | fires once |
| Cycle-stopped | nothing | stands silent (consumer of a stopped writer, ADR 0068) |
| Locked Bang | — | stands silent while locked; fires on the first Tick it is not |

Ticket 02 reduces to tests for each row; the Absence clear already ships.
- **D5 — Superseded by D13.**
- **D6 — Independent Halt activation follows D2 and D3.** With an independent X that could Bang the Halt above `**` over `:P`: X bangs → the Halt locks the `**`, which stands silent, and `:P` is silent. X quiet, suppressed or locked → the Halt is inert, the `**` fires and `:P` plays, with a missed Bang at the Halt. X a dynamic Bang landing after the Halt's Turn → missed Bang at the Halt, the `**` fires.
- **D7 — A Halt activator that depends on the target is a cycle.** `**` → X → Halt → `**` stops all three Expressions under ADR 0068 and diagnoses "same-Tick dependency cycle". `lock_covers` is not extended transitively.
- **D8 — Readers of a standing Bang's Cells wait for it.** The Bang Function reserves its own Span for its clear, with a self-edge exemption stated for the Bang Function (as for an advancing bundle, not a blanket "vacates" flag). A reader sees the cleared Cells if it executed, `**` if it was locked, and the overwrite if one suppressed it. A `**` written this Tick is not a Snapshot root, so a Copy chain carries it in the same Tick. A partial overwrite suppresses the whole Bang Function; the lone `*` left is not a Bang next Tick. Matches Orca's `J` under `*`.
- **D9 — Examples are Grids, not a model.** Each decision is recorded as a small Grid with its edge list, implied Turn order and Tick-by-Tick outcome. These are the red Source Tick tests tickets 01–05 owe. No separate executable model.
- **D10 — Blocked by placement-semantics 03.** Bang tickets 01 and 03 are blocked by placement-semantics ticket 03 (emission into vacated Cells), which needs placement 02.
- **D11 — ADR 0072 is drafted now as Proposed**, carrying D1–D10, and accepted when tickets 01–06 land.
- **D12 — Glossary, edited with the implementation:** Bang (the pulse value), Bang Function (standalone `**`: root-only, intrinsically active, no operands; its Turn activates aligned roots and clears its Span), Missed Bang (a Bang delivered to a root that has taken its Turn); Bang display retired to an _Avoid_ entry.

### Audit against existing behaviour

Three read-only audits checked D1–D12 against the shipped language, under the constraint that the Bang Function is a new Function following existing Function rules and that every other rule emerges from composition. The Bang Function's Turn is today's start-of-Tick fire (`execution.rs:254-266`: blank the Span, activate `bang_roots`) moved into the dependency schedule. Most changes to existing behaviour follow from that move.

- **D13 — No Bang-specific placement edge.** D5 is withdrawn. A mover or emission reaching a `**` has no edge to it beyond the existing ones (the intrinsic-contact skip, `tick.rs:973-979`), so Source order decides, as for a mover train under ADR 0060 rule 3b. `**<<` and a `vv` from the north enter after the `**` fires; `>>**` and a `^^` from the south are blocked and write `**`. This is Orca's reading order. It changes today's mover-into-`**` outcomes for east and south movers (`tick.rs:1646`).
- **D14 — Ticket 05 is dropped.** A Copy with partial input writes nothing (ADR 0069, CONTEXT.md Copy entry); its old `**` fires once (D4).
- **D15 — The missed-Bang diagnostic loses its Selected-only restriction** (`execution.rs:657`). Every Bang delivered to a root that has taken its Turn is diagnosed "Bang reached a root that has taken its Turn", including a static Bang and a Bang reaching a locked root.
- **D16 — Contact activation is removed.** A blocked mover writes `**` and activates nothing; the `**` fires on the next Tick. Keeping contact would fire the contacted root twice.
- **D17 — Emergent consequences accepted**, each an existing rule applied to the new root, with no exception added:
  - A typed `**` in any intrinsically active writer's Output Portal is suppressed by its write (`.+0102` over `**` is silent).
  - A Halt over a `**` locks it; the "not an Expression root" diagnostic no longer applies to `**`.
  - A locked `**` under a Copy is carried every Tick, so it triggers every Tick.
  - A `**` in an Increment's or Interpolation's own Output Portal is a same-Tick cycle (writer before covered, reader after writer).
  - A Copy carrying a Function spelling onto a standing `**` is a refused Function replacement.
  - A partial overwrite suppresses the whole `**`; the stray `*` it leaves is not a Bang.
  - A cycle-stopped `**` is diagnosed "waiting on a same-Tick dependency cycle" each Tick it stands.
  - Collisions sound one Tick later, and collisions with a Comment or the Grid edge now sound.
  - A Directional Bang Function whose mover is blocked on its first step is re-activated by that `**`: after placement-semantics 03 it alternates `vv` and `**`; before it, the pair is a cycle. The Function reference's `*v`, `*<`, `*>` examples change.
  - `lock_covers`'s doc comment (`tick.rs:762-765`) is restated: in D3 the excluded producer is the Halt's only activator, so the lock does not withhold its Turn.

Unavoidably new: a Function whose Turn activates its aligned roots and vacates its own Span (a new declaration modelled on the Advance's own-anchor write site, `portal.rs:461`, with its own self-edge exemption predicate rather than `advances()`), and the parser naming a root `**` as that Function through the existing root-only arm (`parser.rs:234`), keeping `**` out of `from_spelling`. Already shipping, and kept: all-side activation, a Copy reading blanks under a typed `**`, a mover entering a `**` that fires earlier in Source order.

## Examples

Model evidence: each edge list and outcome below is derived by hand from the scheduler rules cited above. Tickets 01–04 owe them as failing Source Tick tests first. Rows are Grid rows; `→` is an ordering edge.

**E1 — D3, Halt above its activating Bang.** `["*!      ", "**      ", "!>007FC4"]`. Edges: `*!` → `**` (lock); `**` → `!>` (Bang). `**` → `*!` is dropped by `lock_covers`. Tick 0: the Halt is inert; the `**` fires and clears; `!>` plays C4; "Bang reached a root that has taken its Turn" at (0,0). Tick 1: nothing. Today: plays, and "`*!` target is not an Expression root".

**E2 — D6, independent activator.** `[".=0101    ", "  *!      ", "  **      ", "  !>007FC4"]`. Edges: `.=` → `*!` (Bang at (0,1), W−2 of the Halt); `*!` → `**` (lock); `**` → `!>`. Every Tick: the Halt locks the `**`, which stands; nothing plays; no diagnostic. With `.=0102`: the Equality clears (0,1), the Halt is inert, the `**` fires, `!>` plays, missed Bang at (2,1); then nothing.

**E3 — D7, activator depending on its target.** `["    =<    ", "  *!  =^  ", "  **=>    ", "  !>007FC4"]`. Edges: `**` → `=>` (reads (2,2)); `=>` → `=^` (writes (6,2), which `=^` reads); `=^` → `=<` (writes (6,0)); `=<` → `*!` (potential Bang at (2,0), north of the Halt); `*!` → `**` (lock). A cycle not covered by `lock_covers`: all five Expressions stop; "same-Tick dependency cycle" at the first cycle computation in Parser order, `=<` at (4,0); `!>` waits. The `**` stands. Nothing could actually carry a Bang round the loop, because `=>` would read the cleared Cells; potential supply orders it anyway (ADR 0032).

**E4 — D1, halted producer.** Tick 0: `["          ", "  .=0101  ", "          ", "  !>007FC4"]` plays and leaves `**` at (2,2). Edit row 0 to `***!      `. Tick 1: `**` (0,0) → `*!` (E+2) → lock `.=`; `.=` → `**` (2,2) (writer before covered), but `.=` is locked and writes nothing; `**` (2,2) fires, `!>` plays. Tick 2: the Halt is inert, the Equality bangs, `!>` plays. Today Tick 1 is silent.

**E5 — D4, invalid producer.** Tick 0: `[".=0101  ", "        ", "!>007FC4"]` plays. Edit to `.=01  `. Tick 1: the Equality is diagnosed and writes nothing; its `**` fires; `!>` plays. Tick 2: nothing.

**E6 — D13, movers in Source order.** `["**<<      ", "!>007FC4  "]`: Tick 0, the `**` fires first, `!>` plays, and `<<` enters: `[" <<       ", ...]`. `[">>**      ", "  !>007FC4"]`: Tick 0, `>>` is blocked and writes `**` over (0,0); the `**` at (2,0) fires, `!>` plays; `["**        ", ...]`. Tick 1: the `**` at (0,0) fires with no aligned root.

**E7 — D8, readers.** `["**      ", "=v      ", "        ", "!>007FC4"]`: `**` → `=v` (reader); the `**` clears, `=v` copies blanks, nothing plays (as today). `[".=0101    ", "  *!      ", "  **      ", "  =v      ", "          ", "  !>007FC4"]`: the `**` is locked every Tick, `=v` carries it to (2,4), `!>` plays every Tick.

**E8 — D17, Output Portal over a typed Bang.** `[".+0102  ", "**      ", "!>007FC4"]`: `.+` → `**` (writer before covered); `03` suppresses it; nothing plays. Today `!>` plays on Tick 0.

**E9 — D16 and D17, first-step block.** `["***v", "    ", "  00"]`. Tick 0: the `**` activates `*v`, which emits `vv` at (2,1). Tick 1: `vv` is blocked by `00` and writes `**`; nothing is activated. Tick 2: `**` (2,1) → `*v`. Before placement-semantics 03 the emitter → occupant edge makes this a cycle; after it, the `**` fires and `*v` emits `vv` again, so row 1 alternates `vv` / `**`.
