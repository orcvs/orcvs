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

The issue `Blocked by` graph is authoritative. Portal retirement, Function locality, parser ownership cleanup, and tracing cleanup can begin independently. Typed operand extraction follows Function locality and strict Raw Play.

This section previously placed the structural parser tickets after the ownership cleanup. They did not wait: issues 07 and 08 shipped while 04 was still open, and in doing so removed the Atom handoff 04 was written to simplify. The `Blocked by` lines, not this paragraph, were authoritative then as now — the ordering was stated here and enforced nowhere.

## Out of scope

- Sequence broadcasting and pervasive evaluation, which remain in the Sequence-values effort.
- Implementing the Language Map, Tick Functions, structural Sequence Functions, or new MIDI output
  Functions.
- Removing or changing `#[inline(always)]` annotations without new before/after benchmark evidence.

## Decisions

Historical parser benchmarks found a measurable benefit from selected forced inlining. Parser and evaluator refactors must retain the annotations unless measurements justify a targeted change, and must record the exact before/after command and results when they claim to move a cost.

A benchmark target now exists and covers the parse paths, so the hedge this section once carried about its absence no longer applies. What it does not supply is a comparison: `mise run bench` reports in the format the CI action's regex needs, which discards criterion's own baseline comparison, and its measurement budget is deliberately coarse enough for a threshold that cannot see below tens of per cent. A local before/after is criterion's saved-baseline comparison at default fidelity on one machine, after a warm-up run — a cold binary reports the Source parse almost three times its settled cost. A refactor that moves a signature rather than generated code owes no measurement at all; it says so on the `Not run` line.

The current Portal type is not an implementation of ADR 0009. Deleting its unused placeholder does
not remove the Portal domain concept or constrain the later Tick Plan destination design.
