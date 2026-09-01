# 06 — Centralize typed operand extraction

**What to build:** Give scalar and terminal Function implementations one strict path for declared
arity, operand type, and stack order, removing repeated pop-and-convert boilerplate without
designing Sequence broadcasting prematurely.

**Blocked by:** 02 — Make Function definitions compiler-checked; 03 — Enforce Raw Play operand contracts.

**Status:** ready-for-agent

- [ ] Arithmetic and Raw Play consume operands through one strict typed extraction seam.
- [ ] The Function signature remains authoritative for arity, operand order, and Number-versus-Note
      expectations.
- [ ] Direct, nested, and Source-driven evaluation report consistent missing-operand and type
      diagnostics.
- [ ] Existing scalar results and terminal Play commands remain behaviorally unchanged.
- [ ] Sequence values, scalar repetition, equal-length pairing, and pervasive failure behavior stay
      out of this ticket.
- [ ] The existing evaluator benchmark workload is run before and after; the exact command and
      results are recorded and measured forced-inlining choices are retained unless the results
      justify a targeted change.
