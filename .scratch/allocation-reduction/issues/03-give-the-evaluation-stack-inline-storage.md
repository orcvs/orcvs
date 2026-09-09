# 03 — Give the evaluation stack inline storage

**What to build:** Evaluating an Expression uses inline storage for the Operand Stack, so an
ordinary Tick allocates nothing at all.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] `Stack::new` stops taking a heap block for every Expression. The path is
      `Interpreter::execute` -> `Context::new(inputs, atoms.len())` -> `Stack::new(limit)`, which is
      `Vec::with_capacity(limit)` over a 24-byte `Value`.
- [ ] The storage is inline with growable overflow, following `05e4490` and `a215af7` — the same
      treatment already applied to the Parser's pending stacks and to the inline Expression records.
      The inline capacity is chosen from the Expressions the Parser actually accepts and the choice
      is recorded here.
- [ ] ADR 0028's requirement still holds: the Operand Stack's depth is proven sufficient for every
      Expression the Parser accepts, or exhausting it answers a diagnostic. Inline storage must not
      turn an exhausted stack into a panic.
- [ ] A Tick over an already-parsed Source allocates nothing for an Expression that fits the inline
      capacity. `a_tick_over_an_already_parsed_source_allocates_per_expression_and_not_per_row` in
      `lang/tests/allocation.rs` is a ceiling and passes either way; strengthen it once zero is true
      and replace its `FINDING (2026-09-09)` comment with a note saying what was corrected.
- [ ] An Expression that exceeds the inline capacity still evaluates correctly, and a test covers
      it.
- [ ] Every existing `lang` test and property passes unchanged, including the `Stack` suite and the
      Interpreter suites.
- [ ] `execute` is the benchmark that covers this path. Note it for the comparison the benchmark
      workflow runs; do not run the comparison locally.

## Comments

What `memory-verification/01` measured:

```
tick over 11 Expressions                   -> 11 blocks,   816 bytes
tick over 44 Expressions                   -> 44 blocks,  3264 bytes
execute [Bang]                             ->  1 block,     24 bytes
execute [Add, 1, 2]                        ->  1 block,     72 bytes
execute [Add, Add, 1, 1, Subtract, 10, 5]  ->  1 block,    168 bytes
```

Exactly one heap block per Expression, sized to its Atom count. A bare `**` Bang — one Atom, no
operands, no arithmetic — still pays for a heap allocation to be evaluated.

This is the most mechanical of the three findings, because the repository has already done it twice
to neighbouring structures. `05e4490` ("Avoid heap allocation for common parser pending stacks") and
`a215af7` ("Keep eight Expression records inline with growable overflow") are the precedent, the
`arrayvec` dependency is already in the workspace manifest with `default-features = false`, and the
shape they established is the shape to follow.

Note the interaction with `lang/src/lib.rs`'s size assertions. The test
`the_answer_seam_is_the_size_the_execute_benchmark_was_measured_against` pins
`size_of::<Performance>()` and `size_of::<Interpretation>()` against the `execute` benchmark's
recorded figures, and its comment explains a 46ns-to-52ns move in terms of those sizes. Inline
storage changes what `Context` and `Stack` cost, not what `Interpretation` answers, so that test
should be unaffected — but if it moves, it is notice rather than a defect, and `execute` is the
measurement to take again.
