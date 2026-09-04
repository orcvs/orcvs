# Specify the Orcvs evaluation machine

**Status:** ready-for-agent

## Goal

Make ADR 0028 true of the code and of the glossary. The ADR names a machine, states what it guarantees, and requires that its instruction set be one declaration everything else derives from. Four of its requirements have no owner: the ADR's own inaccurate statements, the vocabulary it introduces, the operand roles it says must be derived, and the value-or-effect kind its rule implies.

## Why

ADR 0028 was written as a specification, but only one ticket traces to it. `pre-split-defects/15` bounds the Operand Stack, which is the ADR's fifth paragraph. Nothing owns the other four.

Two of the ADR's statements are wrong against the code, and one is weaker than it should be. A specification that misstates the mechanism it is specifying will be followed into the wrong fix, so correcting the text comes before deriving anything from it.

The single-declaration requirement is further along than the ADR says. `lang-foundations/02` made Function definitions compiler-checked and `lang-foundations/06` centralized typed operand extraction, both resolved. What survives those two is narrow and specific: an operand's role and position are still restated in every Function body as a bare index, so transposing Raw Play's channel and velocity compiles silently and no diagnostic fires. That is exactly the failure the ADR names.

The vocabulary gap is plain. `CLAUDE.md` makes `CONTEXT.md` the source of truth for vocabulary. `CONTEXT.md` has 49 glossary entries and none of them is Evaluator, Operand Stack, absence marker, instruction, or Atom. The ADR introduces a machine whose parts have no agreed names.

## Scope

Every claim in these issues was verified against the code on this branch, not taken from the ADR. Where the ADR overstates or misstates, the issue records the smaller, true version and cites the file and line.

## Not in scope

- Bounding the Operand Stack. `pre-split-defects/15` owns it and is ready. Issue 01 here must leave that ticket's proof intact rather than restate it.
- The value model. ADR 0028 leaves the Atom-or-Sequence pair open and ADR 0026 records the uniform-Sequence alternative; that choice belongs to the value model and to `sequence-values`.
- The form of the instruction-set declaration. ADR 0028 constrains only that there is one of it. Issue 04 asks whether derived dispatch is wanted at all before anyone designs a form for it.
