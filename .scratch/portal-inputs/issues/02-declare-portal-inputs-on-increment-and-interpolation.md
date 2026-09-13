# 02 — Declare Portal inputs on Increment and Interpolation

**What to build:** Extend the Function table so Increment `~+` and Interpolation `~>` declare a Portal
input role — ordinary result Portal, Number type — beside their cell operands. Introduce a
validated binding type so Portal inputs participate in the same operand-validation contract as
cell operands, including signature-order diagnostics.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Increment and Interpolation declare one Portal input at the ordinary result Portal, typed as
      Number, in the same declaration table that already states cell operand roles and pervasion.
- [x] A validated Portal input binding exists; callers cannot reach a Function formula with an
      unvalidated Portal spelling.
- [x] Lang tests prove operand shape and type faults precede Portal faults when both operands and
      the Portal spelling are wrong.
- [x] Increment and Interpolation formulas are reachable through the new binding without yet
      rewiring Turn execution (injected Portal values in tests are sufficient for this ticket).

## Comments

The Function table declares an ordinary-result Number Portal input for Increment and
Interpolation. Generated `PortalOperands` and `Context::bind_portal` validate cell operands
through `Stack::extract` before decoding a borrowed Portal spelling into `PortalNumber`.
The validation-order regression failed against the old Previous channel, then passed through
this binding for Note operands and Sequence operands at either signature position.
