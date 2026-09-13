# 03 — Add visible Increment and Interpolation

**What to build:** Implement scalar feedback Functions `~+` and `~>` by reading the previous visible
Number at the ordinary result Portal in the current Source Snapshot.

**Blocked by:** None — every listed blocker is resolved.

**Status:** resolved

**Tags:** release/v1

- [x] Empty Portal initializes as Number `00`.
- [x] Increment returns `(previous + step) % modulus` and rejects zero modulus.
- [x] Interpolation moves toward target without overshoot; rate `00` holds.
- [x] Tests prove that a previous Note, a Note operand, a Sequence, Cells at the Portal that do not
      form a valid Number, or another non-Number operand diagnoses rather than converting implicitly.
- [x] Both Functions remain scalar exceptions to Sequence broadcasting.
- [x] Cross-Tick state is visible only in Source Snapshot Cells.
- [x] `CONTEXT.md` gains a glossary entry for the Increment Function `~+` and the Interpolation
      Function `~>`, naming each spelling, its operands, and the rule that its previous value is read
      from the ordinary result Portal in the current Source Snapshot rather than held as hidden
      state. Glossary text lands with the issue that builds the behaviour, as
      `spatial-tick-planning/01` did for `Turn`, `Producer`, and `Effect`.
- [x] Live Editing and stale-tail behavior remain deterministic.

## Answer

Two more rows in `define_functions!`, still in `lang/src/functions/tick.rs`. The rows are

```
Increment     => ("~+", Value, Scalar, Atom, false, [step: Number, modulus: Number]),
Interpolation => ("~>", Value, Scalar, Atom, false, [rate: Number, target: Number]),
```

and they parse, render, and reach Tick planning the same way Clock did: the spelling table drives
`Function::try_from`. Both bind through `Stack::extract`. The previous is not an operand. It is the
third field `TickInputs` was already written to hold, a language-level `Previous` of Empty, Number,
or Occupied — no Grid, no Source, no hidden store. Empty is Number `00`. Occupied is any other
present Language Unit, including a Portal that does not exist, and both Functions refuse it under
one `PreviousNotNumber` so a Source shown two wordings about one Portal has to work out that they
are the same fault.

`orcvs` fills that field at the Turn. `tick_inputs` reads the ordinary result Portal of this
computation from working Source — values written earlier this Tick included — through
`Token::Number.decode`, the same path `operands()` uses. Two space cells are Empty. A last-row
Function has no Portal and is Occupied rather than clamped. The write half is unchanged: an
ordinary result writes only its current encoding and never clears a stale tail.

### The formulas

Increment is `(previous + step) % modulus` in `u64`. Zero modulus diagnoses as `ZeroWrap`, not as
Modulo's zero-divisor and not as Clock's `ZeroCycle`: Increment is wrapping a running Number, and a
Source shown "cannot count a cycle" or "cannot modulo by zero" would be told about a Function it
did not write. The narrowing back into a Number diagnoses rather than falling back, for the reason
Clock's does.

Interpolation is ADR 0012's three orderings. Each distance is taken only in the branch whose
subtraction is non-negative. Rate `00` holds because a step of nothing is still a step of at most
`rate`, and that is the formula, not a case.

### What is pinned

Lang tests supply previous through `TickInputs`: the wrap enumeration, the literal sequences, the
wider add (`FF + 02` modulo `0F` is `02`, not the byte-wrapped `01`), zero modulus, the three
interpolation orderings, rate `00`, Occupied previous, Note operands, and Sequence refusal at both
operand positions, including that the refusal precedes the wrap check.

Orcvs Tick-by-Tick Grids pin the Portal thread: empty initialises as `00`, two Ticks advance from
what the first wrote, a live-edited previous is the Number the next Tick reads, a stale tail
survives, G4 / `.+` / `0X` / a truncated pair diagnose and write nothing, and a last-row Increment
diagnoses rather than inventing `00`.

### Left unpinned

`TickNumberOutOfRange` has no test, and cannot have one while the formulas are total. Sequence
previous at the Portal of a Number-writing Function is Occupied at the language seam; a Sequence of
Numbers written across the Portal looks like a Number plus a stale tail, which is ADR 0009's
ordinary-result rule rather than a second reading. The anchor half of `TickInputs` still has no
consumer; ADR 0013's Random is the Function that will read it.
