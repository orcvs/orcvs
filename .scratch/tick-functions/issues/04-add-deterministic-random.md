# 04 — Add deterministic Random

**What to build:** Implement `~? seed minimum maximum` with ADR 0013's ChaCha8 seed layout and
inclusive byte-range mapping.

**Blocked by:** None — every listed blocker is resolved.

**Status:** resolved

**Tags:** release/v1

- [x] Reversed bounds normalize and equal bounds return that value.
- [x] Seed, absolute Tick, signed Position coordinates, and Sequence index occupy the specified bytes.
- [x] A fresh ChaCha8 stream supplies the first `u64` for each scalar result.
- [x] Golden vectors pin seed bytes, stream output, and range mapping.
- [x] Each Random reads its own anchor Position, so two Randoms in one Expression differ.
- [x] Moving a Function changes its stream; identical inputs reproduce it.
- [x] Note seed or bound operands diagnose in Random tests rather than converting implicitly.
- [x] Sequence index distinguishes broadcast elements.
- [x] `rand_chacha` is added only to `lang`, with default features disabled and dependency audit.
- [x] Native and `wasm32-unknown-unknown` results match.
- [x] `CONTEXT.md` gains a glossary entry for the Random Function `~?`, naming its spelling, its
      seed/minimum/maximum operands, and the determinism rule that its stream is a function of seed,
      absolute Tick, Position, and Sequence index. Glossary text lands with the issue that builds the
      behaviour, as `spatial-tick-planning/01` did for `Turn`, `Producer`, and `Effect`.

## Comments

This ticket must use the Rust dependency-change workflow in addition to ordinary Rust verification.

### Evaluation carries one anchor per Expression, not one per Function

Issue 01 threaded `lang::TickInputs { tick, anchor }` into interpretation, but
`emit_expression_root` builds one of them from the **Expression root's** anchor and hands it to
`Interpreter::execute` for the whole Expression. A nested Function is therefore told its root's
Position rather than its own.

The Tick is genuinely shared across an Expression, so issues 02 and 03 are unaffected. Random is
the first Function whose result depends on where it sits, and root granularity breaks it: in
`.+~?010010~?010010` both Randoms would be seeded at the root's column and row, draw the same word
from the same ChaCha8 stream, and return the same Number. That contradicts ADR 0013's "Functions at
different Positions have independent reproducible streams" and makes "moving one intentionally
changes its stream" untrue of a nested one.

Widening `TickInputs` does not fix this — there is no per-Function Position anywhere in evaluation
to put in it. The parser hands the Interpreter a flat `Atoms` sequence and an `Atom` carries no
Position, while the Language Map that does know each Language Unit's anchor is not consulted past
the root. So this ticket owns a design decision before it owns an implementation:

- Decide where a Function's own anchor comes from. The candidates are carrying a Position per Atom
  through parsing, having the Interpreter derive one from the root anchor plus each Atom's offset
  in its Expression, or asking the Language Map for the unit at that Span. ADR 0024 records that the
  Map holds spellings rather than Atom types, which bears on the third.
- Record the decision. If it changes what interpretation is given, it amends ADR 0012's input list
  and belongs in an ADR; if it only changes how the anchor is derived, the ticket answer is enough.
- Or establish that root granularity is the intended reading of ADR 0013, in which case say so and
  state what a nested Random is defined to do.

Whichever way it goes, the golden vectors this ticket pins must cover a Random nested inside another
Function, not only a Random at an Expression root — a root-only vector cannot tell the two designs
apart.

## Answer

One more row in `define_functions!`, still in `lang/src/functions/tick.rs`:

```
Random => ("~?", Value, Intrinsic, Pervasive, Elementwise, false, [seed: Number, minimum: Number, maximum: Number]),
```

It parses, renders, and reaches Tick planning the same way Clock did. The body is pervasive like
Clock and binds through `Stack::apply_indexed`, which is `Stack::apply` with the zero-based
Sequence index handed to the element so equal bounds at different positions do not share a stream.
Each scalar result is a fresh `rand_chacha::ChaCha8Rng` seeded from ADR 0013's 32-byte layout
(zero-initialized; explicit seed, little-endian Tick `u64`, column `i64`, row `i64`, Sequence
index `u32`; the last three bytes stay zero). The first `u64` maps into the inclusive width
`u16::from(high) - u16::from(low) + 1`, so `00`–`FF` is 256 rather than a wrapping 0. Reversed
bounds swap; equal bounds return that value. A Note at any operand diagnoses. `rand_chacha` 0.10.0
lives only in `lang`, default features off; its `rand_core` 0.10.1 has no OS-randomness
dependency, so native and wasm32 streams match by construction.

### Per-Function anchors already exist

The comment above is stale. ADR 0032 made each Function a `Computation` with its own
`node.anchor`. `take_turn` already calls `tick_inputs(self.tick, node.anchor)`, so a nested
Random is told its own Position without carrying Position per Atom, without asking the Language
Map for Atom types (ADR 0024), and without amending ADR 0012's input list. This ticket records
that as the answer rather than changing how the anchor is derived.

`.+~?010010~?010010` is the vector that tells the two designs apart. The left Random sits at
column 2 and draws `0A`; the right sits at column 10 and draws `0F`; Add writes `19`. Seeding
both at the Expression root would draw `02` twice and write `04`. Production writes `19`.
