# 06 — Centralize typed operand extraction

**What to build:** Give scalar and terminal Function implementations one strict path for declared
arity, operand type, and stack order, removing repeated pop-and-convert boilerplate without
designing Sequence broadcasting prematurely.

**Blocked by:** 02 — Make Function definitions compiler-checked; 03 — Enforce Raw Play operand contracts.

**Status:** resolved

**Tags:** release/v1

- [x] Arithmetic and Raw Play consume operands through one strict typed extraction seam.
- [x] The Function signature remains authoritative for arity, operand order, and Number-versus-Note
      expectations.
- [x] Direct, nested, and Source-driven evaluation report consistent missing-operand and type
      diagnostics.
- [x] Existing scalar results and terminal Play commands remain behaviorally unchanged.
- [x] Sequence values, scalar repetition, equal-length pairing, and pervasive failure behavior stay
      out of this ticket.
- [x] The existing evaluator benchmark workload is run before and after; the exact command and
      results are recorded and measured forced-inlining choices are retained unless the results
      justify a targeted change.

## Answer

Arithmetic and Raw Play now extract operands through one strict evaluator seam driven by each
Function's canonical signature. The extractor preserves signature order, reports declared arity,
and enforces Number and Note positions before the implementations receive typed values. Existing
forced-inlining annotations remain in place. Behaviour coverage exercises direct and nested
evaluation diagnostics and Source-level incomplete/type diagnostics without adding Sequence
semantics.

The diagnostic contract follows the boundary that can observe the defect. Direct and nested
evaluation share the same typed arity and operand errors. Incomplete Source never reaches the
evaluator: permissive analysis reports its existing `expected a token` syntax diagnostic, while
Source-driven nested type failures report the same typed operand errors as direct evaluation.
Existing coverage for the visible incomplete-Source revision and the strengthened nested Source
type cases pins both sides of that boundary.

Numeric conversions deliberately remain outside this strict extractor. Their documented evaluator
contract accepts either Number or Note for identity/conversion behavior even though their Source
literal signatures are monomorphic; forcing them through the arithmetic/Raw Play seam would change
that behavior. Sequence and pervasive extraction remain deferred.

Benchmark command before and after:

```sh
cargo bench --package lang --bench lang -- execute --output-format bencher
```

- Before: `11 ns/iter (+/- 0)`
- After: `13 ns/iter (+/- 0)`
