# 03 — Remove the Absence Marker

Status: ready-for-agent
Blocked by: 01, 02

**What to build:** Remove `Atom::Empty` and everything that guards it:
`Rendered::Nothing`, `Untyped::from_atom`'s refusal, the parser proptest
guard, the `_` spelling, and the stated-Absence tests. Remove the Absence
Marker from CONTEXT.md and reword the entries for Equality, Delay, Euclidean
and Return. Write the ADR described in `../spec.md`, Decisions.

## Acceptance criteria

- [ ] `Atom` has no variant without a Source encoding.
- [ ] `Encoding::render` cannot answer "nothing".
- [ ] CONTEXT.md and the new ADR describe a Bang Output and no Absence
      Marker.
- [ ] Gates pass with `PROPTEST_CASES=32`, including rustdoc with
      `-D warnings`.
