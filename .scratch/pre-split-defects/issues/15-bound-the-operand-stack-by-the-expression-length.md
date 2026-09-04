# 15 — Bound the Operand Stack by the Expression length

**What to fix:** `Args` is `Stack<16>` while `Atoms` holds `EXP_LEN` of 32. Nothing connects the two
numbers, and `Stack::push` calls `ArrayVec::push`, which panics when it is full. A 64-Cell Source
Expression parses cleanly and then panics the Evaluator.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] An Expression the parser accepts cannot overflow the Operand Stack.
- [ ] The bound is derived from `EXP_LEN` rather than written as a separate number.
- [ ] A comment states why the bound holds, so a later arity change does not silently invalidate it.
- [ ] A regression test parses the reproduction below and asserts that evaluation does not panic.
- [ ] `Stack::push` has no reachable panic left.

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
