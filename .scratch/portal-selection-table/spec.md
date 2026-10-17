# Declare each Read's and Write's address in its Function table row

Status: resolved

## Problem Statement

`PairSelection::Distance` and `PairSelection::Position` carry a `Function`
only so `functions/read.rs` can match on it again to bind the operands that
address the pair. Both matches end in `_ => unreachable!`, held by a comment
saying only `Function::input_portal` and `Function::output` build one, so a
selection naming the wrong Function is a runtime panic rather than a compile
error. Which Reads and Writes select by distance or by Position, and in which
direction, is written again in `selection()` and `input_portal()`, and the
Writes that carry a `value` are listed a third time in `carried!`.

## Solution

- A Read's or Write's `define_functions!` row declares how its operands
  address its pair: `distance: <Portal> <Direction> <n field>` or
  `position: <Portal> <column field> <row field>`, where `<Portal>` is
  `Input` or `Output`.
- The macro generates `DistanceRule` and `PositionRule`, one variant per row
  that declares one. `PairSelection::Distance(DistanceRule)` and
  `PairSelection::Position(PositionRule)` replace the variants that carried a
  `Function`. Each rule's `distance()`/`position()` binds the operands of its
  own row's struct through an exhaustive match, and `DistanceRule::direction`
  answers the direction the row declares.
- `selection()` and `input_portal()` read the declared rule for the Reads and
  Writes. Track, Push and the absolute Copy keep their own selections.
- The macro implements `Carried` for every row with a `value: Untyped` role,
  replacing the hand list.
- A sweep over `Function::ALL` checks that every declared rule binds its own
  row's operands, so a new row is covered by its declaration.

Behaviour-preserving: every selection resolves to the same pair and
diagnoses as before.

## Decisions (Toby, 2026-10-17)

- Build candidate 2 of the fifth architecture pass.

## Out of scope

- Track's, Push's and the absolute Copy's selections, which carry no
  `Function` and have no `unreachable!`.
- The Interpreter's dispatch, which ADR 0028 accepts.
