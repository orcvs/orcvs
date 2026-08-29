# 03 — Wrap general arithmetic over bytes

**What to build:** General arithmetic accepts Numbers, returns Numbers, and wraps modulo 256 across
the full `00`–`FF` range instead of saturating. Notes diagnose unless explicitly converted. Division
by zero diagnoses instead of returning zero, as ADRs 0011 and 0021 require.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] Addition, Subtraction, and Multiplication wrap modulo 256 rather than saturating or clamping.
- [ ] Subtraction is ordered `left - right` and wraps rather than clamping at zero.
- [ ] Division by zero produces a diagnostic and no result.
- [ ] General arithmetic rejects Note operands rather than changing its result type or modulus.
- [ ] The saturation tests are replaced by wrapping tests.

## Comments

`lang/src/functions/math.rs` currently uses `saturating_add`/`saturating_mul`, clamps subtraction at
zero, and returns `Atom::Number(0)` for divide-by-zero under a comment calling that "terribly
incorrect". `test_add_saturates_at_255_on_overflow` and `test_multiply_saturates_at_255_on_overflow`
assert the behaviour ADR 0011 contradicts, so they change with the implementation.
