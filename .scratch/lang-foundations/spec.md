# Prepare `lang` for the documented Function roadmap

**Status:** ready-for-agent

## Goal

Make the current scalar parser and evaluator safe to extend before the remaining Function families,
Sequences, and the Language Map land. Preserve the measured hot-path choices while removing
uncoordinated Function metadata, loose operand coercion, and parser invariants that the compiler
cannot enforce.

## Required behavior

- One compiler-checked definition supplies each real Function's spelling and operand signature.
- Function parsing, rendering, enumeration, and dispatch cannot silently disagree.
- Raw Play enforces its documented Number, Number, and Note operands without broad byte coercion.
- Parser inputs are immutable and parsed storage moves without redundant reconstruction.
- Expression syntax and parsed values cannot become desynchronized.
- Lenient Source analysis and strict parsing use structural outcomes rather than sentinel Functions.
- Test-only tracing support does not enlarge the shipped dependency graph.

## Delivery

The issue `Blocked by` graph is authoritative. Portal retirement, Function locality, parser
ownership cleanup, and tracing cleanup can begin independently. Typed operand extraction follows
Function locality and strict Raw Play; the structural parser tickets follow the ownership cleanup.

## Out of scope

- Sequence broadcasting and pervasive evaluation, which remain in the Sequence-values effort.
- Implementing the Language Map, Tick Functions, structural Sequence Functions, or new MIDI output
  Functions.
- Removing or changing `#[inline(always)]` annotations without new before/after benchmark evidence.

## Decisions

Historical parser benchmarks found a measurable benefit from selected forced inlining. Absence of a
benchmark target on the current branch is not evidence that those annotations were unmeasured.
Parser and evaluator refactors must record the exact before/after benchmark command and results, and
must retain the annotations unless measurements justify a targeted change.

The current Portal type is not an implementation of ADR 0009. Deleting its unused placeholder does
not remove the Portal domain concept or constrain the later Tick Plan destination design.
