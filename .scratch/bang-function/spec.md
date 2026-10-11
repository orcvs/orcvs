# A standalone Bang is a Function

Status: ready-for-agent

## Problem Statement

A performer expects visible Bangs and predictable activation. To let a typed `**` fire while a `**` a Tick wrote does not fire twice, Source keeps a hidden record of where the previous Tick wrote Bangs, and that record gets things wrong:

- A lone `*` typed beside a written Bang makes the parser read the pair one Cell to the west. The record does not match, so a Bang nobody typed fires.
- A Bang written into an operand is forgotten after one quiet Tick. Freeing it later fires or not depending only on how long the performer waited.
- Two Grids that look identical behave differently, and a Source saved and reloaded fires a Bang the live one would not. ADR 0003 says the Source Snapshot is the complete language state; the record breaks that, and a test pins the breakage.

## Solution

A standalone `**` in Source is the **Bang Function**, as Orca's `*` is an operator. It is a new Function that follows the rules every Function follows: no operands, root-only, intrinsically active, can emit Bang. Its Turn does what every standalone `**` already does before the first Turn — activate its aligned roots and clear its two Cells — inside the dependency schedule instead of before it. Every other outcome emerges from existing rules applied to that root; nothing is remembered between Ticks.

The decisions are D1–D17 in `issues/07-settle-clearing-and-halt-precedence.md`, with worked Grids E1–E9. The language decision is `docs/adr/0072-a-standalone-bang-is-a-function.md` (Proposed). The historical walkthrough (`lang/bang-function-prototype.html`) is not authoritative.

## User Stories

1. As a performer, I want a `**` I type, paste or load where no writer covers it to fire its aligned roots once on the next Tick, so that I can trigger a Function by hand.
2. As a performer, I want a Source I save and reopen at the same absolute Tick to behave exactly as before, so that saving never changes the performance.
3. As a performer, I want two identical Source Snapshots at the same absolute Tick to behave the same, so that what I see is what runs.
4. As a performer, I want Equality, Delay and Euclidean to trigger their aligned roots in the Tick they bang and leave `**` visible for one Tick, so that a Note and its Bang land together and I can see the pulse.
5. As a performer, I want a producer's next Turn to suppress the `**` it left, by banging again or clearing, so that a pulse fires once.
6. As a performer, I want a Bang carried down a chain of Copies to arrive in the same Tick and fire once.
7. As a performer, I want a lone `*` typed beside a written Bang never to trigger anything and to stay where I typed it.
8. As a performer, I want a Bang written into an operand to do nothing while it stays there and to fire once whenever I free it.
9. As a performer, I accept that a `**` whose writer takes no Turn, is invalid, is removed, or writes elsewhere fires once more, because the language keeps no hidden memory (D1, D4).
10. As a performer, I want a typed `**` to activate roots on every side of it.
11. As a performer, I want a blocked Self-Banging Function to leave `**` that triggers every aligned root on the next Tick, so that collisions with anything, including a Comment or the Grid edge, make sound (D16).
12. As a performer, I want a mover meeting a `**` to follow Source order, as it meets a mover train: one later in Source order enters after the `**` fires, one earlier is blocked (D13).
13. As a performer, I want a Halt above a `**` that activates it to report the missed Bang, and a Halt activated independently to lock the `**`, which stands silent until unlocked (D3, D6).
14. As a performer, I want a Directional Bang Function above a typed `**` to be activated by it and emit into the Cells it leaves.
15. As a performer, I want the glossary to name the standalone `**` the Bang Function and the pulse value Bang.
16. As a developer, I want Tick planning to take only the Grid, the Cells, the Language Map and the Tick, and test helpers to plan exactly as production does.
17. As a developer, I want no Bang-specific state to survive a commit and no Bang-specific scheduling rule.

## Implementation Decisions

- **Declaration.** A new Function-table row, `**`: no operands, Intrinsic activation, can emit Bang, root-only. Its write site is its own Span, modelled on the Advance's own-anchor write site (`orcvs/src/source/portal.rs:461`); its Turn activates `bang_roots` over its Span and writes a vacating clear (`WriteKind::Vacate`). This is today's start-of-Tick fire (`orcvs/src/source/tick/execution.rs:254-266`) as a Turn. A new kind or bundle is needed: no existing Interpretation both activates and clears.
- **Self-edge exemption** by a predicate stating the declaration clears its own Span, not by widening `advances()`, which also drives contact, the contact skip and `reserves_over`.
- **Parsing.** The parser's root-only `Some("**")` arm (`lang/src/parser.rs:234`) names the Bang Function at an Expression start. `from_spelling` does not resolve `**`, so `**` in an untyped operand stays the Bang value and in a typed operand stays invalid syntax. Sweep tests over every Function (`parser.rs:638-646`, `1745-1779`) and the Function reference example test (`console/src/function_reference.rs:972-1000`) account for it.
- **Removed:** the provenance record (`Source.bang_display`, `shared_bang_display`, `forget_bang_display`, the commit update, `TickPlan.bang_display`, `PlanningSnapshot.bang_display`, the Bang bookkeeping in `resolve`, the extra planning and execution argument); the start-of-Tick Bang clear; contact activation (`contacted_roots`, its use in `activate`, the contact arm in execution).
- **Missed-Bang diagnostic** loses its Selected-only condition (`execution.rs:657`).
- **No new ordering edge.** Overwrite, reader, activation, lock, `lock_covers`, writers-first and the intrinsic-contact skip apply unchanged.
- **Documentation.** Accept ADR 0072 and set the superseded ADRs' status lines. CONTEXT.md: split Bang into the value and the Bang Function, add Missed Bang, retire Bang display to an _Avoid_ entry; amend Source Snapshot, Self-Banging Function, Halt Function and Producer; amend the `ActivationSource` and `takes_no_operand` docs in `lang/src/atom.rs`. Restate `lock_covers`'s doc comment (`orcvs/src/source/tick.rs:762-765`).

## Testing Decisions

- One seam: the Source Tick API, driving a Source through edits and Ticks and asserting Play Commands, diagnostics and committed Grids after each step. No test asserts on scheduler internals.
- E1–E9 in ticket 07 become failing tests first.
- The two provenance reproductions become regression tests: `***` plays nothing and the typed `*` survives; a freed operand `**` fires exactly once, with zero or more quiet Ticks before the edit.
- Tests pinning removed or changed behaviour flip at their sites: the saved-display reload test, the typed-over-display and block-edit-over-display tests, `a_planned_tick_fires_a_typed_bang_once_and_never_the_display_a_tick_wrote`, the mover contact tests, the Directional Bang emission-block tests (`tick.rs:1703-1723`), the odd-gap mover test (`tick.rs:1646`), the Halt-over-`**` diagnostic (`tick.rs:2870-2895`), and the Function reference settled-area test.
- Each D17 consequence has a test. Locally, `PROPTEST_CASES=32`.

## Out of Scope

- Persistent provenance or destination history to prevent surviving-Bang replay.
- Bang-specific ordering to make mover occupancy independent of direction.
- Console presentation of Bangs beyond what the Language Map already paints.

## Tickets

- 01 — A standalone `**` is a Bang Function
- 02 — Producers that take no Turn leave their Bang to fire
- 03 — A mover meets a Bang in Source order
- 04 — A Halt above a typed Bang misses it
- 05 — dropped (D14)
- 06 — Close the remaining PR #213 review findings
- 07 — Settle ordered clearing and Halt precedence (resolved)
