# 03: Remove the Absence Marker

**What to build:** Every value Function answers a value. The Absence Marker
leaves the language, along with each guard that keeps it out of operands, the
parser and rendering, its `_` display, and the tests that state it as an
answer. CONTEXT.md drops its entry and rewords Equality, Delay, Euclidean and
Return around the clear. A new ADR records the decision, as described in
`../spec.md`, Decisions.

**Blocked by:** 01, 02

**Status:** resolved

- [x] Every value the language holds has a Source encoding.
- [x] Rendering an answer cannot yield "nothing".
- [x] CONTEXT.md and the new ADR describe the clear and no Absence Marker.
- [x] Gates pass with `PROPTEST_CASES=32`, including rustdoc with
      `-D warnings`.

## Comments

2026-10-17: `Atom::Empty`, `Rendered`, `Function::answers_only_bang` and `BangAnswer::Only` are removed, `Untyped::from_atom` carries every Atom, and ADR 0073 records the decision. CONTEXT.md replaces the Absence Marker entry with Clear.
