# 03 — Retire the Sequence value

Status: needs-triage
Blocked by: 01

## Work

Implement the removal half of
[ADR 0063](../../../docs/adr/0063-a-list-is-cells-in-a-claim-not-a-value.md).
Remove the Sequence value, the Sequence Functions (Range, Note Range, Reverse,
Concatenate, Select, Replace), pervasive extension and the `:` family. Retire
the Sequence entries in `CONTEXT.md` and the Sequence column of the Function
reference.

## Acceptance

- [ ] No Function answers or accepts a Sequence; the Function table has no
      pervasion or Sequence-width column left to declare.
- [ ] Reservation reserves one Atom's Cells for every value Function; the
      variable-width path ADR 0036 added is gone.
- [ ] Every `:` spelling parses as an unknown spelling.
- [ ] A chord plays from several Timed Play roots activated by one Bang, under
      a test that asserts every note of it.
- [ ] `CONTEXT.md` (Sequence, Atomic Function, Range, Reverse, Concatenate,
      Select, Replace, and the Sequence clauses of other entries) and the
      Function reference describe the language without the value.
- [ ] No test or doctest still constructs a Sequence.

Large mechanical change across `lang` and `orcvs`. Run the scoped gates for
both, and the workspace gates once before the pull request.
