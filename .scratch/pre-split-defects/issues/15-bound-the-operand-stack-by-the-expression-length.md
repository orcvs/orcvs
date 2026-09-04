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

`Args` is now `Stack<EXP_LEN>`, declared beside the proof the comments above set out. `EXP_LEN` is
chosen; the stack's size is derived from it. The comment states the proof in the form that survives
an arity change: no Atom raises the depth by more than one, because a literal pushes one value and a
Function pushes one value only after popping the operands its signature declares. Peak depth
therefore never exceeds the Atom count. Stated that way it also covers a nullary Function, which
raises the depth by one exactly as a literal does, so nothing an arity change can do invalidates it.

The Atom count is bounded by the type rather than by the caller, which the comment now says. `Atoms`
is an `ArrayVec<Atom, EXP_LEN>`, so `Expression` cannot record more and `Parser` diagnoses the
attempt as `SyntaxError::ExpressionTooLong`. Because `Interpreter::execute` is public and takes
`&Atoms`, a caller who never went through the parser is held to the same bound. Naming that
dependency is the point: were `Atoms` ever to become a `Vec`, it would contradict a premise the
comment states rather than one it left unsaid.

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

Three tests cover the bound, and they do not cover it equally.

`interpreter::test::an_expression_at_the_parser_bound_evaluates_without_exhausting_the_operand_stack`
parses the 64-Cell reproduction and asserts the whole answer, which is `!> 0F 01 C4`: the chain sums
fifteen of the sixteen literals into the channel and leaves the sixteenth as the velocity.

`interpreter::test::every_chain_the_parser_accepts_evaluates_without_exhausting_the_operand_stack`
enumerates every left-leaning chain length, under every root Function and over every binary Value
Function the chain can be built from, and asserts that none answers `OperandStackExhausted`. It
names no spelling: the chain link is derived from `Function::ALL` like the root, so a Function
respelled later stays covered. Its non-vacuity guard pins depth rather than Atom count. A root of
arity `a` over a chain of `k` binary Functions is `2k + a + 1` Atoms and peaks at `a + k`, so the
deepest walk `EXP_LEN` admits is `(EXP_LEN + a - 1) / 2` for the widest `a` the definitions declare
— seventeen today — and the test asserts it reached exactly that. Atom count alone was too weak:
`.^` and `.v` also reach `EXP_LEN` Atoms, at a depth of sixteen that the old stack would have held,
so the flag could have been satisfied while the one discriminating case was lost.

`interpreter::property::evaluating_every_expression_the_parser_accepts_returns_rather_than_panicking`
generates whole Expressions of any shape from the Function definitions and makes the same
assertion. Review measured the first version of it as ~98% vacuous: 252 of 256 cases refused by the
parser, 0 reaching `EXP_LEN` Atoms, peak depth 5. Two causes, both fixed. `literal_source` ignored
the `Token` its position declared, so a Note in a Number position and a Number in a Note position
were refused as Source text before the Evaluator saw them; it now takes the `Token`. And the chain
length was drawn uniformly from `0..EXP_LEN`, which put nearly all the mass past the parser's bound
before anything was built around it; it is now short most of the time, with a minority running the
full range so the bound is straddled rather than only exceeded. The runner is driven directly rather
than through `proptest!`, so what the cases reached is counted across them and asserted at the end:
a majority must reach the Evaluator, and the deepest walk must exceed the widest signature, which no
flat Expression can do. Measured after the fix, over five runs at the default 256 cases: 233–241
evaluated, deepest walk 13–16. At the pull-request tier's 32 cases: 24–29 evaluated, deepest 6–14.

The property still cannot catch this particular regression, and the module docstring now says so
rather than claiming otherwise. Depth seventeen needs one narrow shape — an arity-three root whose
first operand is a chain of exactly fourteen and whose other two operands are literals — which is
about 1 in 40,000 of what the generator draws. Reverting `Args` to `Stack<16>` fails the two
deterministic tests and not the property. The division is deliberate: the enumeration owns the tight
boundary, and the property owns the breadth of shape an enumeration of one shape cannot reach.

`interpreter::test::peak_depth` models the walk from the signatures, because the depth a walk
reaches is not something the Evaluator reports. Where evaluation would stop early the model keeps
walking, so its answer is an upper bound; for the chain shapes whose depth is asserted the peak
falls while literals are still being pushed, before any Function has run, so model and machine agree
exactly there.

`stack::test::an_exhausted_operand_stack_diagnoses_rather_than_panicking` covers the new error path
over a two-slot stack, so it states the behaviour without depending on the size `Args` chooses.

ADR 0028's fifth paragraph described the defect as live. It now states the derived bound and its
proof, and keeps the two-target analysis of what the panic would have cost, which is why the push
answers rather than asserts.

The frame-size arithmetic in the note above was not measured. `mise run bench` was not run: three
agents share this machine, so nothing measured here would have a trustworthy baseline. No claim in
the change depends on the frame size.

Review measured the frame cost the note above only estimated: `Value` is 24 bytes, `Stack<16>` is
392 and `Stack<32>` is 776, so the Operand Stack grows by 384 bytes rather than the estimated 512 to
1024. `Stack::extract`'s inner buffer grows 136 to 264 bytes, which the estimate did not account
for. These are the reviewer's measurements, not mine, and no claim in the change depends on them.

Review also confirmed that the deepest walk any parser-accepted Source can produce today is exactly
seventeen, so `Stack<EXP_LEN>` carries fifteen slots the current Function set cannot use. That is
the arity independence the third checkbox asked for rather than slack to reclaim.

ADR 0028's fifth paragraph needed three further corrections after the first rewrite. It said a
Function's net change is "never positive for any declared arity", which is false at arity zero and
strictly stronger than the code's own comment — in a sentence whose whole point is that a Function
added later needs no new proof. It said "the same Expression" of a 64-Cell Expression the rewrite
had moved to the end of the paragraph, leaving the reference pointing forward at nothing. And it
kept "no third option exists" after dropping the enumeration of the two that do. All three are
fixed, and the paragraph now also records that `Atoms` carries the capacity in its type, so a caller
who reaches `Interpreter::execute` without going through the parser is held to the same bound.
