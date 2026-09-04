# 03 — Declare each operand's role with its signature

**What to build:** Make an operand's role part of the single Function declaration, so reading the wrong operand is a compile error rather than a silent transposition.

**Blocked by:** None — can start immediately.

**Status:** resolved

**Sources of truth:** ADR 0028 requires each operand's role name to be derived from the single declaration; `lang-foundations/02` established `define_functions!`; `lang-foundations/06` established `Stack::extract`.

- [x] A Function's operand roles are declared once, beside its spelling, kind, and operand types.
- [x] A Function body names the operand it reads rather than indexing it by position.
- [x] Transposing two same-typed operands of one Function fails to compile, or fails a test that names both roles.
- [x] `Operands::number` and `Operands::note` retain the property that a mistyped read is unreachable, which `Stack::extract` already guarantees.
- [x] Raw Play's channel, velocity, and note are each named at the point they are read.
- [x] The evaluator benchmark workload is run before and after; the exact command and results are recorded, and measured forced-inlining choices are retained unless the results justify a change.

## Comments

The defect is visible in `lang/src/functions/mod.rs`. Raw Play reads `operands.number(0)` as channel and `operands.number(1)` as velocity. Both positions declare `Token::Number`, so `Stack::extract` cannot tell them apart and neither can the compiler. Swapping the two lines produces a Function that validates a velocity against the channel domain and a channel against the data-byte domain, compiles clean, and passes every type and arity test. `play_rejects_channels_outside_the_midi_range` would catch this particular swap because the two domains differ, but a Function whose same-typed operands share a domain has nothing to catch it. `lang/src/functions/math.rs` has nine of those: every non-commutative one — `subtract_impl`, `divide_impl`, `modulo_impl` — depends on `number(0)` and `number(1)` being in signature order and says so nowhere.

ADR 0028 names this precisely: "each operand's position lives in the body of the Function that reads it", and it is "why an operand can be reordered without a diagnostic". The right-to-left walk paragraph explains what is at stake — the walk order exists so operands reach a Function in signature order, and "reversing it would silently transpose every operand of every non-commutative Function". The same silence applies one level down, inside the body.

The signature already carries arity, order, and type. Adding the role is a small extension of a declaration that exists rather than a new mechanism. `Operands` is `pub(crate)` and its two accessors have three call-site shapes in total, so the blast radius is `lang/src/stack.rs`, `math.rs`, and `mod.rs`.

The `Operands` inner `ArrayVec<Atom, EXP_LEN>` is sized independently of any signature. It is not the Operand Stack and does not carry `pre-split-defects/15`'s defect — the longest signature is three — but whatever this ticket does to `Operands` should not introduce a second unproven bound.
