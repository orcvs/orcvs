# 07 — Make the Comment a Parser unit

**What to build:** Move the Comment rule out of the byte pre-pass and into the parse. The Parser
recognizes `##` and claims the remaining Cells of its row as one Comment unit. The claimed Cells
are not operands.

**Blocked by:** None.

**Status:** ready-for-agent

**Tags:** release/v1

## Why this exists

`source_end` (`orcvs/src/source/language_map.rs:504`) runs before the Parser. It finds the first
`##` with `row.windows(2)`, which tests every overlapping byte pair rather than stepping in
two-Cell units. `walk_row` then cuts the row there, so the Parser never sees the Comment.

The doc comment states the rule that makes the unaligned scan safe: *"No Language Unit spelling
holds a `#`, so nothing the walk reads can begin before the introducer and end after it."*

The Note Range `:#` of issue 05 makes that false. An Expression can begin at any column, so `:#`
at columns 3 and 4 with a `#` at column 5 makes the pair at columns 4 and 5 read as `##`. The walk
then cuts the row at column 4, in the middle of the Function. A user can reach that state with one
keystroke during a Live Edit.

The property suite repeats the same scan rather than reading the implementation
(`language_map.rs:1405-1409`), and its doc comment repeats the same premise
(`language_map.rs:1355-1360`). So the property agrees with the defect and would not report it.
Both must change together.

ADR 0033 partitions a row by parse. The Comment is the last exception to that rule; this ticket
removes it.

- [ ] The Parser recognizes the `##` introducer and claims the rest of the row as one Comment.
      The claim is a Cell claim, not an operand list: Comment text is arbitrary and is never
      decoded, typed, or evaluated.
- [ ] `source_end` and its pre-pass are deleted. `walk_row` hands the Parser the whole row.
- [ ] `LanguageUnitKind` gains a Comment variant, so the Comment has a Span and an anchor like
      every other unit. It is named `LanguageUnitKind::Comment` today at
      `orcvs/src/source/language_map.rs:56-62`.
- [ ] A Comment answers no value and no effect, and it is never scheduled. It does not become a
      Function and it does not need a third `FunctionKind`.
- [ ] `##` starts a Comment only where a new Expression could start. A `##` inside the
      arity-determined claim of a Function is an operand Cell of that Function, which fails to
      bind and diagnoses. Record this as the decided reading of "partition by parse".
- [ ] One `#` alone stays incomplete or invalid Source, as `CONTEXT.md:151-153` states.
- [ ] `comments_and_live_edit_fragments_do_not_form_language_units`
      (`orcvs/src/source/language_map.rs:981`) is rewritten, because a Comment now does form a
      unit. Its name changes with it.
- [ ] The partition property stops re-deriving the Comment with `windows(2)`
      (`language_map.rs:1405-1409`) and reads the Comment unit the Map reports. The doc comment at
      `language_map.rs:1355-1360` loses the "no spelling contains a `#`" premise.
- [ ] A Tick-by-Tick test covers `:#` next to a `#`, and `:#` next to a real `##` introducer.
- [ ] `CONTEXT.md` states that a Comment is a Language Unit that is excluded from evaluation,
      rather than text the walk removes before parsing.

## Comments

2026-09-09: Raised by the release-candidate audit against `sequence-values/05`. Modelling the
Comment as a Function with a Grid-length parameter list was considered and rejected: operand
counts are fixed for each Function in `define_functions!`, `MAX_OPERANDS` derives from that table
at compile time and holds operands inline, operands are typed through `Token`, and a Function must
answer a value or an effect. A Cell claim carries none of those costs.
