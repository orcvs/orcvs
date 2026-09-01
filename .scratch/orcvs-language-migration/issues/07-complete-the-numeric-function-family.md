# 07 — Complete the numeric Function family

**What to build:** Add Absolute Difference `.|`, Modulo `.%`, Minimum `.<`, Maximum `.>`, and
Equality `.=` with the Number-only contracts from ADR 0011. Equality produces Bang for equal
operands and no value for unequal operands.

**Blocked by:** 01 — Move arithmetic onto the `.` family; 02 — Free Bang spelling; 03 — Wrap general arithmetic over bytes; lang-foundations/06.

**Status:** ready-for-agent

- [ ] Every Function parses and round-trips through its behavior-first spelling.
- [ ] Absolute Difference is symmetric and cannot underflow.
- [ ] Modulo diagnoses a zero divisor and produces no result.
- [ ] Minimum and Maximum return one Number.
- [ ] Equality returns Bang only for equal Numbers and otherwise returns no value.
- [ ] Notes and other Atom types diagnose rather than coercing.
- [ ] Sequence broadcasting is left to the Sequence-values effort.

## Comments

This completes ADR 0011 without pulling first-class Sequence implementation into the migration.
