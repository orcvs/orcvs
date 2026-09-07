# 09 — Decide what an incomplete Function claims

**What to build:** Decide whether an Expression whose operands are not all typed claims every Cell its root
Function's arity declares, only the Cells that hold characters, or one of those plus the first
declared Cell no character reached. Record the answer here and in the ADR
issue 08 writes, before `10` rebuilds the scheduler on it.

**Blocked by:** cell-indexed-parse/06 — Pin the open question under every candidate rule.

**Status:** ready-for-human

**Tags:** release/v1

- [ ] The answer states whether `!>00`, typed at columns 0 to 3, claims columns 0 to 7 or 0 to 3.
- [ ] The answer states what a `**` written at columns 4 and 5 then does: a rejected typed operand,
      or a Bang that activates a root.
- [ ] The answer states whether a half-typed root reports anything, and through what, when a write
      lands on a slot it declared.
- [ ] `8e7bdce`'s failure is re-tested under the chosen rule. The `!>00` case must not fail silently
      again, whichever way the claim is decided.
- [ ] The chosen rule is recorded in the ADR issue 08 writes, beside the two space rules.
- [ ] The answer accounts for option C, which is the rule `8e7bdce` implemented and which the two
      original options both discard.
- [ ] `10`'s scope is settled: whether `8e7bdce`'s slot filter is dropped, kept, or replaced.

## The question

Issue 08 makes an Expression a fixed-width spatial footprint. The root Function's arity sets the
width, and a space inside that width is an unfilled operand rather than a terminator. That is settled.

What 08 does not settle is whether the same rule holds while the operands are still being typed.
`!>` has a four-slot layout — Function, Number, Number, Note — so `!>00` declares eight Cells of
layout from four Cells of Source. `Expression::layout` already emits all four slots for it; the
branch test `layout_preserves_invalid_and_missing_slots_and_later_nested_operands`
(`lang/src/parser.rs:293`) pins that.

So: are the four undeclared Cells claimed, or are they free?

## Why this is not already answered

`8e7bdce` ("Stop a half-typed Function claiming slots its run cannot reach") answered it with the run
rule, which issue 08 deletes.

Its bug was concrete, and its commit message records it: a Bang landing on a half-typed Function's
missing slot read as a write to a typed operand rather than as an activation. The Play root the Bang
was for went silent on every Tick, "with no diagnostic anywhere, because the half-typed root is
terminal and unactivated and takes no turn from which to report anything." A `!>00` four columns to
the left of a producer stopped the music.

`8e7bdce` fixed it by filtering the declared slots down to those abutting the run:

```rust
.filter(|(offset, _, _)| *offset <= span_width)
```

Under a fixed-width partition `span_width` is the parse's own width, so the filter either does
nothing or does the wrong thing. The protection is gone and the failure returns.

Issue 06 closes the other escape. A `**` rejected in a typed Function operand position activates no
root, so a Bang landing in a claimed slot cannot fall back to activating.

## Options

### A — An incomplete Function claims its full arity footprint

- `!>00` claims columns 0 to 7. A `**` at columns 4 and 5 is its second Number operand, is rejected
  as syntax, and activates nothing.
- **Constraint:** consistent with 08's claimed-extent rule and with how ports work in a spatial
  language. An Expression's footprint does not change as its operands are filled.
- **Tradeoff:** typing a Function reserves the Cells its operands will occupy, so a neighbour cannot
  use them. `8e7bdce`'s failure returns unless a diagnostic is added, and the reason it was silent —
  a terminal unactivated root takes no turn — still holds, so the diagnostic cannot come from the
  root's turn. It has to come from the Language Map or from schedule construction.

### B — An incomplete Function claims only the Cells that hold characters

- `!>00` claims columns 0 to 3. A `**` at columns 4 and 5 is a separate Language Unit and activates
  normally.
- **Constraint:** preserves `8e7bdce`'s behaviour without the run.
- **Tradeoff:** an Expression's footprint then depends on whether its operands are filled, which
  contradicts the rule 08 establishes. It also reintroduces the question 08 removes — a half-typed
  Function's extent would again be decided by something other than arity, and a result written into
  a declared slot would complete an Expression that did not claim that Cell.

### C — An incomplete Function claims the typed Cells plus the first declared Cell no character reached

- `!>00` claims columns 0 to 5. A `**` at columns 4 and 5 is its second Number operand, is rejected as
  syntax, and activates nothing. A `**` at columns 6 and 7 is a separate Language Unit.
- **This is what `8e7bdce` actually implemented.** Its filter compares with `<=`, not `<`, so it kept
  the first missing slot deliberately: a write abutting the run could complete the Function, and a
  result landing further out could not. Neither A nor B preserves that.
- **Constraint:** keeps the one capability the commit was protecting — completing a half-typed
  Function by writing into the operand next to it — without reserving Cells no write can reach.
- **Tradeoff:** the claim is neither arity nor typed Cells, so an Expression's extent has a third
  rule of its own. Whether that is a language people can hold in their heads is the question.

## Recommendation

Take **A**, and add the diagnostic.

The harm `8e7bdce` documents was not that the Cells were claimed. It was that nothing said they were.
A claimed slot with no occupant is ordinary live-edit state and should be visible as one, which is a
Language Map diagnostic rather than a Tick outcome — the Tick is exactly where it cannot be reported.

Option B keeps one behaviour at the cost of the invariant that makes the rest of this effort
coherent. Two rules for an Expression's extent is the defect this effort exists to remove.

This is a language decision, so it is `ready-for-human` rather than `ready-for-agent`.

## Comments

Raised while planning the branch rebuild. `10` drops `4d6c17c` outright, because its premise is
purely the run. `8e7bdce` is the one commit on the branch whose premise is false under 08 but whose
*bug* survives, so it needs an answer rather than a deletion.

`08` is absorbed by `cell-indexed-parse/01 — Partition a row by parse, not by whitespace`, so the
ADR this issue's answer belongs in is the one that ticket writes. `cell-indexed-parse/06` makes all
three rules expressible at one place and pins `8e7bdce`'s failure under each, so the decision can be
taken against tested behaviour rather than against prose.

Option C was found by reading `8e7bdce`'s filter operator rather than its commit message. The two
options above were written from the message, which describes the intent but not the `<=`.
