# 06 — Project a Sequence result into Source

**What to build:** Let a Sequence result reach Source again, and let a signature name an operand
that is not a Number or a Note. Both are prerequisites of issues 03 and 05, and until now neither
had an owner.

**Blocked by:** None.

**Status:** ready-for-agent

**Tags:** release/v1

## Why this exists

Issue 04 delivered Portal writes for a complete Sequence result. The dependency-scheduled executor
that replaced it does not use them. Two gaps remain:

**The executor refuses every Sequence result.** `orcvs/src/source/tick.rs:310-321` diagnoses
`Sequence result {..} has no fixed scalar scheduling footprint` and then continues, so it plans no
write at all. The rule below it is the reason: scheduling reserves one scalar Cell pair for each
destination, and `tick.rs:327-336` refuses any encoding whose length is not 2. A variable-width
result cannot use those dependency edges. `Portal::ordinary_result` is now reached only from the
`#[cfg(test)]` helper `plan_result` in `orcvs/src/source/model.rs`. Four spec files defer this
work — `tick-execution-order`, `live-typed-execution`, `cell-indexed-parse`, and this effort — and
no issue owned it.

**A signature cannot name a generic operand.** `lang/src/expression.rs:30-37` gives `Token` six
variants, and `check_token` (`lang/src/stack.rs:624-630`) matches only `Number` and `Note` before
it reaches `unreachable!("scalar and terminal signatures contain only typed operands")`. The four
structural Functions in issue 03 stay generic over Atom type, and Concatenate and Replace take a
Sequence operand. Neither shape can be declared today.

- [ ] A Sequence result reaches its destination Cells. The refusal at
      `orcvs/src/source/tick.rs:310-321` is replaced, not merely relaxed.
- [ ] Scheduling reserves the Cells a variable-width result occupies, so a dependency edge still
      names every Cell the write touches. The scalar Cell pair stays the special case, not the
      only case.
- [ ] A write that does not fit its row or its Grid writes no Cell of it. The complete-fit rule of
      issue 04 holds for a Sequence exactly as it does for a scalar.
- [ ] Two producers whose Sequence results overlap resolve Cell by Cell, as two scalar producers
      already do.
- [ ] `Token` can name a generic Atom operand and a Sequence operand. `check_token`'s
      `unreachable!` is replaced by a decision for each new variant.
- [ ] `MAX_OPERANDS` and the inline operand storage are unchanged in shape. A wider operand type
      does not make the operand list heap-allocated.
- [ ] Tick-by-Tick Source Grid tests cover a Sequence result at a row edge, at a Grid edge, and
      against a competing writer.

## Comments

2026-09-09: Raised by the release-candidate audit. Issues 03 and 05 each carry an acceptance item
that this work must satisfy first, and both listed only resolved blockers, so both read as ready
when neither was.
