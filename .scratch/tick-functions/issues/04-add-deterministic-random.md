# 04 — Add deterministic Random

**What to build:** Implement `~? seed minimum maximum` with ADR 0013's ChaCha8 seed layout and
inclusive byte-range mapping.

**Blocked by:** 01 — Thread Tick and Position into interpretation; sequence-values/02; lang-foundations/06.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Reversed bounds normalize and equal bounds return that value.
- [ ] Seed, absolute Tick, signed Position coordinates, and Sequence index occupy the specified bytes.
- [ ] A fresh ChaCha8 stream supplies the first `u64` for each scalar result.
- [ ] Golden vectors pin seed bytes, stream output, and range mapping.
- [ ] Each Random reads its own anchor Position, so two Randoms in one Expression differ.
- [ ] Moving a Function changes its stream; identical inputs reproduce it.
- [ ] Note seed or bound operands diagnose in Random tests rather than converting implicitly.
- [ ] Sequence index distinguishes broadcast elements.
- [ ] `rand_chacha` is added only to `lang`, with default features disabled and dependency audit.
- [ ] Native and `wasm32-unknown-unknown` results match.
- [ ] `CONTEXT.md` gains a glossary entry for the Random Function `~?`, naming its spelling, its
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
