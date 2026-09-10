# 02 — Add Clock, Delay, and Euclidean Functions

**What to build:** Implement `~.`, `~*`, and `~%` from ADR 0012 using explicit absolute Tick and
Number operands.

**Blocked by:** None — every listed blocker is resolved.

**Status:** resolved

**Tags:** release/v1

- [x] Clock returns `floor(Tick / rate) % modulus`.
- [x] Delay Bangs exactly when `Tick % (rate * modulus) == 0`, including Tick `0`.
- [x] Zero rate or modulus diagnoses and the cycle product cannot byte-wrap.
- [x] Euclidean follows ADR 0012's formula and phase exactly.
- [x] Euclidean handles zero hits, full hits, zero steps, and hits greater than steps.
- [x] Note operands diagnose in Clock, Delay, and Euclidean tests rather than converting implicitly.
- [x] Sequence operands follow the ordinary broadcasting rules once available.
- [x] Tick-by-Tick tests use explicit Source Grids and diagnostics.
- [x] `CONTEXT.md` gains a glossary entry for the Clock Function `~.`, the Delay Function `~*`, and
      the Euclidean Function `~%`, naming each spelling, its operands, and its `_Avoid_` terms.
      Glossary text lands with the issue that builds the behaviour, as `spatial-tick-planning/01`
      did for `Turn`, `Producer`, and `Effect`.

## Answer

Three rows in `define_functions!` and one new module, `lang/src/functions/tick.rs`. The rows are

```
Clock     => ("~.", Value, Pervasive, false, [rate: Number, modulus: Number]),
Delay     => ("~*", Value, Pervasive, true,  [rate: Number, modulus: Number]),
Euclidean => ("~%", Value, Pervasive, true,  [hits: Number, steps: Number]),
```

and nothing else was needed to make them Source: the spelling table drives `Function::try_from`,
which is the only thing the Parser and the Language Map ask, so `~.` parses, renders, and reaches
Tick planning by being declared. Each body reads `ctx.inputs.tick()` once, before it touches the
Operand Stack, and states the formula for one element — the Tick is shared by the whole operation
because an Expression is evaluated at one Tick, and the operands are not.

The formulas are ADR 0012's, evaluated in `u64` throughout. Clock is `(tick / rate) % modulus`, and
the step narrows back into a Number at the end because it is below `modulus`, which is a byte. The
narrowing is a checked conversion that diagnoses rather than one with a fallback Number: `00` is the
first step of every cycle, so a fallback would answer a step no reader could tell from a counted one
if the proof ever broke, and a panic is not available under the Source write guard. Delay is
`tick.is_multiple_of(rate * modulus)` — the same test the ADR spells `Tick % (rate * modulus) == 0`,
differing only at a zero divisor, which validation has already refused. Euclidean reduces the Tick
before the phase offset, exactly as written.

### The `predicate` seam, and why it is where Delay and Euclidean went

Delay and Euclidean each answer a pulse or nothing. `Atom::Empty` is refused as a Sequence member
by `Sequence::new` because it has no Source encoding, so an element-wise map has nothing to write
at a position where an element did not Bang — the reasoning `Stack::predicate` and the Absence
Marker glossary entry already record for Equality. Both therefore go through `Stack::predicate`,
which binds and asks every broadcast element independently at the same Tick, which is what ADR 0012
requires of a broadcast element, and answers one whole-value `Bang` or `Empty` about all of them.
Clock answers a Number and uses `Stack::apply` like every other Atomic Function.

`Stack::predicate` was `F: Fn(O) -> bool` and is now `F: Fn(O) -> Result<bool, Error>`;
`math::equality` became `Ok(left == right)` and is the one caller that can never fail. The
interesting part was the ordering property the old body carries: *every element is still bound once
the answer is settled*, because a bind is where a declared domain is checked, and stopping at the
first `false` would make whether a later element diagnoses depend on which earlier element
disagreed. That property is about the boolean, and it is unchanged. A diagnostic is deliberately
not held to it: the first faulting element in signature order diagnoses the complete operation and
the elements after it are neither bound nor asked. That is not a new rule — it is exactly what
`Stack::apply` already does, and holding a fault to the boolean's rule would mean deciding which of
two faults wins, which is a choice with no reader-visible justification. The doc comment on
`predicate` now says both halves.

No alternative seam was seriously in play. A third method beside `apply` and `predicate` would have
had the same body as `predicate` with a different name, and a Function body that mapped a Sequence
itself would be a second broadcast mechanism free to disagree with the first.

### Diagnostics

Three new `InterpretationError` variants, and deliberately not five:

- `ZeroCycle { function: Function, role: &'static str }` covers the zero rate, the zero modulus,
  and the zero step count. They are one fault — a cycle with no length — under three role names,
  and the variant carries the declared `Function` rather than a spelling written down beside it, so
  the message renders the Cells the Source actually holds: `~. cannot count a cycle with a zero
  rate`. This is `MidiDataByte`'s shape, which shares one diagnostic across the roles occupying the
  MIDI data-byte domain, and it achieves what `DivisionByZero` and `ModuloByZero` achieve by being
  separate variants for a family of two. Neither of those is reused.
- `EuclideanOverfull { hits, steps }` names both operands, because either Cell pair is the one to
  edit. It carries no `Function` field, since only one Function can raise it, but it still renders
  the spelling `~%` from the declaration rather than the word "Euclidean": a Source shown two
  messages about the Cells it wrote should not have to work out that they name one Function.
- `ClockStepOutOfRange { step }` is the diagnostic Clock's narrowing raises. It is unreachable while
  the formula stands, and it is a variant rather than a fallback because the fallback is what would
  be unreadable.

Clock and Delay share one `cycle_factors` helper, and all three Functions build the diagnostic
through one `zero_cycle` constructor, so the three cannot drift apart on what a zero means or on
what it is called, and the rate is answered before the modulus — signature order, which is the order
`Stack::checked` walks operands and `Operands::from_operands` binds them in. Euclidean asks for a
positive step count *before* it compares hits to steps, so `~% 00 00` is a cycle with no positions
rather than a pattern with no onsets; that ordering has a test of its own.

### Dead-code attributes

`Context::inputs` no longer carries `#[expect(dead_code, …)]` — reading the Tick made it an error,
which is what `expect` was there to do — and its doc comment now says the seam has consumers and
that the anchor is still waiting for ADR 0013's Random. `lang/src/tick.rs`'s module doc no longer
says "None of them exists yet". `Pervasion::Scalar` and `Stack::extract`'s suppressions are
untouched: Increment and Interpolation are still unbuilt, and they are the exception those seams
exist to let the table state.

`equality_is_the_only_function_that_can_emit_bang` became
`exactly_the_pulse_answering_functions_declare_that_they_can_emit_bang`, over
`[Delay, Equality, Euclidean]`. Its sibling in `interpreter.rs`,
`only_a_function_that_declares_it_ever_answers_with_bang`, needed no edit and now checks the two new
declarations against what the Interpreter answers, which is the half that would catch a `false` in
the table.

### What is pinned

Nineteen tests in `lang/src/functions/tick.rs`. Each Function is claimed twice over, because the two
kinds of test answer different questions. Clock and Delay are enumerated over every rate and modulus
from `01` to `08` across 256 Ticks against ADR 0012's expression retyped, which pins operand order
and the width the arithmetic is done in and cannot pin the shape of the expression, since a retyped
reference agrees with the body by construction. Beside each sweep is a hand-written statement of
what the operands mean: `~. 02 04` is the literal step sequence `00 00 01 01 02 02 03 03` walked
three cycles over, `~. 80 03` is checked at the Ticks its steps change across a 384-Tick cycle, and
`~* 03 02` Bangs at Ticks `0`, `6`, `12`, `18` and at no Tick between, with a 384-Tick pair beside
it. Euclidean was written this way from the start: checked Tick by Tick against hand-written
patterns (`X..X..X.` and six others) over three whole cycles, which is what pins the reduction
rather than only the first cycle. Then the byte-wrap cases
(`~* 10 20` behaving as 512 and `~* 10 11` as 272, with every multiple of the wrapped 16 asserted
silent); zero hits, full hits, `(00, 00)`, hits greater than steps; Note operands at all six operand
positions; and the broadcasting rules including a whole-value answer, a vacuous empty Sequence, an
element fault diagnosing the complete operation, and incompatible non-scalar lengths.

Two Tick-by-Tick tests in `orcvs/src/source/tick.rs` prove the threading end to end.
`the_tick_functions_answer_about_the_absolute_tick_they_are_planned_at` plans one Grid holding all
three Functions at Tick 1 and Tick 4 and asserts the literal Cell writes; it was checked to fail
when `tick_inputs` is severed to `Tick::ZERO`, which is what `tick-functions/01` asked this ticket
to pin. `a_tick_function_with_no_cycle_diagnoses_and_writes_nothing` asserts both diagnostic
messages reach the Tick Plan and that nothing is written.

### Left unpinned

The anchor half of `TickInputs` still has no consumer; ADR 0013's Random is the Function that will
read it, and `tick-functions/04` carries the per-root granularity analysis it needs.
`ClockStepOutOfRange` has no test, and cannot have one while the formula is total: the step is a
remainder of a modulus that arrived as a Number, so no operands reach it. What it buys is the
failure mode it replaces — a formula change that broke the proof now costs the Expression its Cell
write and says so, rather than answering `00`. No property test was added: the formulas are cheap
enough to enumerate exhaustively over the ranges that matter, which is a stronger claim than a
sampled one.

`Stack::predicate`'s whole-value answer for Delay and Euclidean imports ADR 0011's Equality rule
into ADR 0012, which states nothing about it. `CONTEXT.md`'s Atomic Function and Absence Marker
entries now record it as the rule for every pulse-answering Function rather than as Equality's, but
no ADR says so normatively. That is a language-design decision for the repository owner and is
deliberately left open here.
