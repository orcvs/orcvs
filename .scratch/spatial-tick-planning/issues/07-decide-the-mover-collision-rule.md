# 07 — Decide the Mover Collision Rule

**What to build:** A settled rule for what happens when two Self-Banging Functions want the same
Cells, recorded in ADR 0006. The rule that ships today was chosen to fix a defect without changing
semantics; this decides whether it is the rule the language keeps.

**Blocked by:** None (can start immediately).

**Status:** ready-for-human

**Sources of truth:** ADR 0006 states the move, its refusal, and root contact; ADR 0020 orders Tick
effects by Source position; ADR 0031 evaluates Turns against working Source; ADR 0018 states how a
run of `*` Cells is read; `CONTEXT.md` defines Self-Banging Function and Bang.

## The rule that ships

Two movers approaching each other close the gap by two Cells a Tick, so the parity of that gap never
changes and there are two cases. Source order decides both, and each blocked mover takes ADR 0006's
ordinary refusal — it replaces its current Span with `**`.

```text
>> <<     odd gap, one Cell of overlap    ->  " >>**"  then "  >> "
>><<      even gap, flush                 ->  " ****"  then "     "
```

The odd case always worked and matches Orca, which resolves the same event by evaluation order. The
even case rejected the Tick as a dependency cycle until `order_turns` learned to drop the edge that
would order a mover ahead of a mover earlier in Source.

## The alternative

Both movers are consumed, and one Bang anchors at the first Cell they both want.

```text
>> <<     ->  "  **"        <<  the conflicted Cell is 2; the Bang is [2,3]
>><<      ->  " ** "        <<  the conflicted Cells are 1 and 2; the Bang is [1,2]
```

This is the reading that treats the collision as a write conflict rather than as two independent
refusals, and it says something the shipping rule does not: at an even gap the two movers did not
fail separately, they wanted the same two Cells.

It was not taken now because it needs two Turns to depend on each other before either runs. ADR 0031
runs one Turn at a time against working Source, and a Turn reads Cells rather than another
Function's intent. That makes this a change to the evaluation machine, not a rule inside it.

## Rejected outright

Writing a Bang at each destination Span and letting them overlap. That spells `***`, and ADR 0018
already reads `***` as one Bang plus one invalid `*` — `map.bangs().count()` is 1. Which Cell becomes
litter follows the parse, not the collision. `****` is the spelling that means two whole Bangs.

- [ ] ADR 0006 states the collision rule outright, rather than leaving it to be derived from the
      refusal sentence and the schedule. A reader should not have to run the Grid to learn what two
      movers do.
- [ ] The decision covers the one-Cell overlap of an odd gap and not only the flush pair. The two
      cases differ in how many Cells are contested, and the shipping rule gives one a survivor and
      the other none.
- [ ] The decision covers three or more movers converging. Today they resolve in Source order with
      no cycle, because every kept edge between two movers points from the earlier to the later:
      `">>>>  <<    "` gives `"** >><<     "` and then `"   ****     "`.
- [ ] The decision covers a Directional Bang emission that contests Cells with a mover. An emission
      never vacates its own Span, so "both are consumed" has no meaning for it, and any rule that
      consumes both needs an answer for the pair that is not symmetric.
- [ ] The decision states what a mover blocked by a static Cell does. That case keeps the current
      refusal under either answer, so choosing the alternative leaves the language with two refusal
      rules and a reader needs to know which applies when.
- [ ] If the rule changes, ADR 0006's "replaces its current Span with `**`" is amended rather than
      contradicted, and `two_moves_that_want_the_same_cells_each_bang_in_their_own_span` is rewritten
      to the new rule rather than deleted.
