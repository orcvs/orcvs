# 03 — Wrap general arithmetic over bytes

**What to build:** General arithmetic accepts Numbers, returns Numbers, and wraps modulo 256 across
the full `00`–`FF` range instead of saturating. Notes diagnose unless explicitly converted. Division
by zero diagnoses instead of returning zero, as ADRs 0011 and 0021 require.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Addition, Subtraction, and Multiplication wrap modulo 256 rather than saturating or clamping.
- [x] Subtraction is ordered `left - right` and wraps rather than clamping at zero.
- [x] Division by zero produces a diagnostic and no result.
- [x] General arithmetic rejects Note operands rather than changing its result type or modulus.
- [x] The saturation tests are replaced by wrapping tests.

## Comments

`lang/src/functions/math.rs` currently uses `saturating_add`/`saturating_mul`, clamps subtraction at
zero, and returns `Atom::Number(0)` for divide-by-zero under a comment calling that "terribly
incorrect". `test_add_saturates_at_255_on_overflow` and `test_multiply_saturates_at_255_on_overflow`
assert the behaviour ADR 0011 contradicts, so they change with the implementation.

Implemented byte wrapping for Addition, ordered Subtraction, and Multiplication; made Division by
zero return an interpreter diagnostic; and introduced a Number-only stack conversion for general
arithmetic. Tests cover boundary examples, Note rejection, zero division, and every pair of byte
operands. Verified with both crate scoped gates, `mise run check_wasm`, and `mise run check`.
