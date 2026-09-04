# 15 — Bound the Operand Stack by the Expression length

**What to fix:** `Args` is `Stack<16>` while `Atoms` holds `EXP_LEN` of 32. Nothing connects the two
numbers, and `Stack::push` calls `ArrayVec::push`, which panics when it is full. A 64-Cell Source
Expression parses cleanly and then panics the Evaluator.

**Status:** resolved

**Tags:** release/v1

- [x] An Expression the parser accepts cannot overflow the Operand Stack.
- [x] The bound is derived from `EXP_LEN` rather than written as a separate number.
- [x] A comment states why the bound holds, so a later arity change does not silently invalidate it.
- [x] A regression test parses the reproduction below and asserts that evaluation does not panic.
- [x] `Stack::push` has no reachable panic left.

## Comments

`lang/src/interpreter.rs:6` declares `pub type Args = Stack<16>`. `lang/src/lib.rs:22` declares
`pub const EXP_LEN: usize = 32`. `lang/src/stack.rs:57` pushes with `ArrayVec::push`.

This Source Expression is 64 Cells. It parses to 32 Atoms and panics with `CapacityError` at
`lang/src/stack.rs:57`:

```
!>.+.+.+.+.+.+.+.+.+.+.+.+.+.+01010101010101010101010101010101C4
```

Sixteen slots hold a left-leaning chain of binary Functions exactly, which is why the defect stayed
hidden. Raw Play has arity three, so two operand results already sit on the stack while the chain
fills the rest. Replace `:=` in `sequence-values/issues/03` also has arity three and answers a value,
so the defect becomes easier to reach as soon as that lands.

The bound is provable rather than empirical, and the proof is short. A literal Atom pushes one value.
A Function pops at least one operand and pushes one value, so its net change is never positive. Peak
depth therefore never exceeds the Atom count, and the parser already bounds that at `EXP_LEN`. So
`Stack<EXP_LEN>` cannot overflow for any arity, now or later, and no new proof is needed when a
Function's arity grows.

The cost is stack frame size. A `Value` is roughly 32 bytes, so the frame grows from about 512 bytes
to about 1 KB. That is arithmetic and not a measurement; `lang/benches/lang.rs` is where to check it
if the frame matters.

The severity comes from where the Evaluator runs. `Source::plan_tick` calls `Interpreter::execute`
inside a Tick, under the `SourceCommander` write guard, while Playback continues. Issue 02 in this
effort describes what one panic there does to the lock.

The defect predates the crate split and every current branch. `EXP_LEN` and `Stack<16>` are identical
on `main`, and `Stack<16>` dates from the commit named `modularise`.

ADR 0028 states the rule this violates: every bound the machine relies on is proven or diagnosed,
and never assumed.

`Args` is now `Stack<EXP_LEN>`, declared beside the proof the comments above set out, so the
Operand Stack carries the parser's own bound rather than a second number. The comment states the
proof in the form that survives an arity change: no Atom raises the depth by more than one, because
a literal pushes one value and a Function pushes one value only after popping the operands its
signature declares. Peak depth therefore never exceeds the Atom count, which the parser bounds at
`EXP_LEN`. Stated that way it also covers a nullary Function, which raises the depth by one like a
literal, so nothing an arity change can do invalidates it.

`Stack::push` answers `Result<(), Error>` over `ArrayVec::try_push`, with a new
`InterpretationError::OperandStackExhausted { capacity }`. `Interpreter::execute` propagates it with
`?`; nothing else in the crate pushes outside a test. The diagnostic sits in `InterpretationError`
rather than in `SyntaxError`, because `SyntaxError::ExpressionTooLong` is what the parser answers
about Source text, and this is what the machine answers about its own state. The last checkbox is
the reason the answer exists at all: the proof states what is true today, and a panic in the push
would be the cost of the day it stopped being true, paid inside a Tick under the Source write guard.

`Stack::extract`'s inner `ArrayVec<Atom, N>` still uses `ArrayVec::push`. That one is proven by the
comment already above it — one Atom per successful pop, and a pop yields one only while the stack
holds a value — so it cannot outgrow the stack it drains.

Three tests cover the bound.
`interpreter::test::an_expression_at_the_parser_bound_evaluates_without_exhausting_the_operand_stack`
parses the 64-Cell reproduction and asserts the whole answer, which is
`!> 0F 01 C4`: the chain sums fifteen of the sixteen literals into the channel and leaves the
sixteenth as the velocity.
`interpreter::test::every_chain_the_parser_accepts_evaluates_without_exhausting_the_operand_stack`
enumerates every left-leaning chain length under every root Function, asserts that none answers
`OperandStackExhausted`, and asserts that the enumeration reached the parser's bound at least once
so it cannot go vacuous. Both fail against `Stack<16>`; the second names the case,
`Play over a chain of 14 exhausted the Operand Stack`.
`interpreter::property::evaluating_every_expression_the_parser_accepts_returns_rather_than_panicking`
generates whole Expressions from the Function definitions and makes the same assertion for breadth.
The tight boundary is one narrow shape — only an arity-three root over a fourteen-long chain reaches
depth seventeen within `EXP_LEN` Atoms — so sampling misses it, which is exactly why the enumeration
sits beside the property rather than being replaced by it.

`stack::test::an_exhausted_operand_stack_diagnoses_rather_than_panicking` covers the new error path
over a two-slot stack, so it states the behaviour without depending on the size `Args` chooses.

ADR 0028's fifth paragraph described the defect as live. It now states the derived bound and its
proof, and keeps the two-target analysis of what the panic would have cost, which is why the push
answers rather than asserts.

The frame-size arithmetic in the note above was not measured. `mise run bench` was not run: three
agents share this machine, so nothing measured here would have a trustworthy baseline. No claim in
the change depends on the frame size.
