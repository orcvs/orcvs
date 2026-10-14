# Give every Function one Output declaration

Status: open

## Problem Statement

CONTEXT.md says every Function has one Output Portal, and that what a Function does there is one of four things: a value answer writes there, Halt locks the root there, a Terminal Output Function writes nothing there, and a Source-writing Function writes its declared Source effect instead. `lang` answers that with five separate predicates: `performs_terminal_output`, `source_effect`, `locks_root`, `output_portal` and `dynamic_output_portal`. `output_portal()` returns `None` by asking three of the others.

Each consumer rebuilds the order of precedence by hand:

- `orcvs/src/source/portal.rs`, `PortalAccess::resolve`, runs an if-chain: terminal, then Source effect, then static Portal, then lock inside it.
- `orcvs/src/source/language_map.rs`, `output_portal_reservation`, excludes `locks_root` before it asks `output_portal`.
- `lang/src/atom.rs`, `DECLARED_CHANGES`, joins three Output predicates with `||` in its Write row.
- `lang/src/interpreter.rs` asks `locks_root` and then `source_effect` before dispatch.
- `tick/dependencies.rs` and `tick/execution.rs` ask `source_effect` and `dynamic_output_portal` separately.

A new effect kind falls through each chain instead of failing to compile.

## Solution

`lang` declares `Output`, a total sum type, and `Function::output() -> Output` answers it from the private `FunctionKind` and the Portal declarations:

- `Static(PortalCoords)`: a value answer written at a fixed offset.
- `Dynamic(PairSelection)`: the pair the operands select at the Turn.
- `Source(SourceEffect)`: the declared Source write.
- `Lock(PortalCoords)`: Halt's root lock.
- `Terminal`: a Play Command, with no Cell destination and so no coordinates.

There is no `None` variant: every Function falls into exactly one of these. The narrow predicates `performs_terminal_output`, `source_effect`, `locks_root`, `output_portal` and `dynamic_output_portal` are removed, so every consumer matches on `Output`. `answers_value` stays as the broad question ADR 0029 asks callers to use.

## Decisions

- The narrow predicates are removed, not kept as wrappers (Toby, 2026-10-14).
- `output()` is total; there is no `None` variant (Toby, 2026-10-14).
- Variant names follow CONTEXT.md's Output Portal entry; `Terminal` carries no coordinates, and the entry gains a clause saying a Terminal Output Function's Portal is never addressed (Toby, 2026-10-14).
- The change is strictly behaviour-preserving. Tests that asked a removed predicate ask `output()` instead with equivalent assertions.

## Testing Decisions

- The existing Portal sweep over `Function::ALL` in `lang/src/atom.rs` asserts each Function's `Output`.
- Every Tick, Halt, Bang, Source-write, schedule-reuse and replacement test passes, unchanged except where it named a removed predicate.

## Out of Scope

- The interpreter's `unreachable!` arm, which ADR 0028 accepts.
- Input Portals. `input_portal()` stays as it is.
- Candidate 2, declaring Portal selection in the Function table.
