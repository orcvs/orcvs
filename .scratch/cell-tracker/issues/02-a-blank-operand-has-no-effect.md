# 02 — A blank operand has no effect

Status: needs-triage

## Work

Implement [ADR 0062](../../../docs/adr/0062-a-blank-operand-has-no-effect.md).
An operand whose Cells are all empty makes its Function do nothing that Tick,
with no diagnostic. A partly empty operand stays malformed.

## Acceptance

- [ ] A value Function with a blank operand answers the Absence Marker, plans no
      write and raises no diagnostic.
- [ ] A Terminal Output Function with a blank operand, activated by a Bang,
      emits no Play Command and raises no diagnostic.
- [ ] A partly empty operand such as `" 0"` still diagnoses.
- [ ] A blank Return leaves its enclosing operand blank, so the enclosing
      Function does nothing (after 01, or tested once 01 lands).
- [ ] Boundary tests cover every operand type: Number, Note, Bang-receiving
      roots and Terminal Output operands.
- [ ] `CONTEXT.md` and the Function reference agree with the behaviour.
