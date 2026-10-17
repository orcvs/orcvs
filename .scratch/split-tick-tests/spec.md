# Split tick.rs's test module by topic

Status: resolved

## Problem Statement

`orcvs/src/source/tick.rs` was about 6,200 lines, of which about 5,350 were
its test module and two property modules. It is the most-edited file in the
workspace, so every effort read and merged around tests unrelated to its
change, while the Function families (read, write, push, track, nested_copy,
bang_function, absolute_copy) already live in topic files beside it.

## Solution

- Each cluster of the test module moves to its own `#[cfg(test)]` file under
  `tick/`: absence, bang_activation, conflicts, copy, cycles, halt,
  live_editing, movers, nesting, outputs, play_order, replacement, schedule
  and tick_functions. The property modules move beside the examples they
  generalise: Cell-wise conflicts into `conflicts.rs`, nested Expressions
  into `nesting.rs`.
- The helpers more than one topic uses move unchanged to `tick/fixture.rs`;
  the replacement helpers stay in `replacement.rs`, the one topic that uses
  them.
- `tick.rs` keeps only its shipped code.

Test-only and behaviour-preserving: the same 1720 tests run, by name.

## Decisions (Toby, 2026-10-17)

- Build candidate 6 of the fifth architecture pass.

## Out of scope

- Merging `fixture.rs` into `observed.rs`. The two share names with
  different meanings (`rows_of`, `turns`, `diagnostic`) and seed Sources
  differently, so one dialect is a rewrite of the tests, not a move.
- `schedule_reuse.rs`'s helpers: its `source_of` writes one revision, which
  is what those tests check, so it does not duplicate `observed::source_of`.
