# 07 — Make the Comment a Parser unit

**What to build:** Respell the Comment introducer `##` as `||`, and move the Comment rule out of
the byte pre-pass and into the parse. The Parser recognizes `||` and claims the remaining Cells of
its row as one Comment unit. The claimed Cells are not operands.

**Blocked by:** None.

**Status:** resolved

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

**The spelling changes at the same time, and for a separate reason.** ADR 0023 gave `#` two jobs:
`:#` names Note Range and `##` opened a Comment. `#` is the notation the musical domain reaches
for first, and the Comment is the one unit with no musical meaning. ADR 0035 records the decision
and the alternatives that were refused. Doing both in one commit is deliberate: the spelling
currently lives in seven places that this ticket deletes or rewrites anyway, so changing it on the
way through touches each site once instead of twice, in the right order.

## Spelling

`||`. `|` is not a sigil and appears in exactly one spelling, `.|` Absolute Difference, as a
second Cell. A doubled glyph whose single form is not a sigil is the operandless family — `**`
Bang and the `^^ vv << >>` Activations — which is where a Comment belongs. `!!` was refused
because `!` is the Terminal Output sigil and ADR 0004's Halt, Directional Bang, and Jump are the
operandless Functions that family will need; `//` was refused because `/` is the octave-zero
marker in Note spelling. ADR 0035 carries the full reasoning.

One `|` alone is incomplete or invalid Source, exactly as one `#` was.

## Shape

The Parser already decides how far each unit's claim reaches, and already answers that question
three ways at one point in `take_language_unit` (`lang/src/parser.rs:186-198`). A Comment is a
fourth answer — claim the rest of the Source — recognized in the same place, before the
`(Token, Atom)` match, because it yields no Atom:

- recognize `||`, consume the remainder with `next_token(self.source.len())`, and
- `add_positioned(Token::Comment, None, cell_start..self.start + self.consumed(), parent)`.

`Token::Comment` and no Atom, rather than an `Atom::Comment`. An `Atom` is a fixed-width `Copy`
value with no text-carrying variant, so it cannot hold a row; and the properties that assert Atoms
render back to the exact Source Cells they claim could never hold for one that could.

## Checklist

- [x] The Parser recognizes the `||` introducer and claims the rest of the row as one Comment.
      The claim is a Cell claim, not an operand list: Comment text is arbitrary and is never
      decoded, typed, or evaluated.
- [x] `Token` gains a `Comment` variant. `Atom` gains nothing — a Comment records no Atom, so
      `lang`'s value vocabulary (Sequence membership, `Display`, the interpreter) is untouched.
- [x] `source_end` and its pre-pass are deleted. `walk_row` hands the Parser the whole row.
- [x] `LanguageUnitKind` gains a Comment variant, so the Comment has a Span and an anchor like
      every other unit. It is named `LanguageUnitKind::Comment` today at
      `orcvs/src/source/language_map.rs:56-62`.
- [x] `name_units` (`language_map.rs:545`) keys the unit kind off `entry.token` rather than
      `entry.atom`, or a Comment's absent Atom falls into the diagnose branch and every Cell of it
      is reported as an unmatched character. Kind is a syntactic fact and `Token` is where syntax
      lives; every other unit's token and atom already agree, so nothing else changes.
- [x] `Glyph` gains a Comment variant and `From<Token> for Glyph` (`orcvs/src/glyph.rs:51`) maps
      it. `record_expression` fills `self.glyphs` from `entry.token` across the whole positioned
      run, so the entire Comment renders with it. `console` and `shell` need a colour for it.
- [x] A Comment answers no value and no effect, and it is never scheduled. It does not become a
      Function and it does not need a third `FunctionKind`. Confirm both hold structurally rather
      than by an added rule: `expression.atoms()` collects `Option<Atom>` and short-circuits on the
      Comment's `None`, and `function_candidate` (`language_map.rs:427-432`) matches only
      `(Token::Function, Atom::Function(_))`, so `root` is `None` and the schedulers at
      `tick.rs:789,811,1125,1143` never see it.
- [x] `||` starts a Comment only where a new Expression could start. A `||` inside the
      arity-determined claim of a Function is an operand Cell of that Function, which fails to
      bind and diagnoses. Record this as the decided reading of "partition by parse".
- [x] One `|` alone stays incomplete or invalid Source, as `CONTEXT.md:151-153` states of `#`.

### Tests and properties

- [x] `comments_and_live_edit_fragments_do_not_form_language_units`
      (`orcvs/src/source/language_map.rs:981`) is rewritten, because a Comment now does form a
      unit. Its name changes with it.
- [x] The partition property stops re-deriving the Comment with `windows(2)`
      (`language_map.rs:1405-1409`) and reads the Comment unit the Map reports. The doc comment at
      `language_map.rs:1355-1360` loses the "no spelling contains a `#`" premise.
- [x] The same property asserts every Language Unit is exactly two Cells
      (`language_map.rs:1378`). A Comment is row-length. Restate it as two Cells unless Comment.
- [x] `lang/src/parser.rs:1270-1272` expects that *"a complete analysis holds only complete
      entries"* and unwraps the Atoms. A Comment analysis is complete and holds no Atoms, so this
      panics rather than failing an assertion. Restate the premise: a complete analysis holds only
      complete entries, or is a Comment.
- [x] `language_map.rs:1496` asserts `atoms().is_some() == diagnostic.is_none()`, and
      `language_map.rs:1502-1511` that the Atoms render back to the Source Cells they claim. A
      Comment has no diagnostic, no Atoms, and unrenderable text. Write the exclusion as the
      property's premise — *a Comment is a complete Language Unit that is not a value* — rather
      than as a skipped case. That sentence is the load-bearing statement of this design.
- [x] `parser.rs:1256` (`atoms().is_some() == entries.len() == expression.len()`) holds unchanged,
      because a Comment record carries no Atom. Confirm rather than edit.
- [x] The two proptest generators (`parser.rs:1132-1133`, `language_map.rs:1313-1314`) and the two
      coverage checks (`parser.rs:1387`, `language_map.rs:1599`) generate and count `||` and a lone
      `|` instead of `##` and a lone `#`.
- [x] A Tick-by-Tick test covers `.|` next to a `|`, and `.|` next to a real `||` introducer —
      the aligned successors of the `:#` case that motivated this ticket.
- [x] A Tick-by-Tick test covers `:#` next to a `#`, which is now ordinary Source with no Comment
      in it at all.

### Documents

- [x] `CONTEXT.md:151-153` gives the Comment the `||` spelling and states that it is a Language
      Unit excluded from evaluation, rather than text the walk removes before parsing.
- [x] ADR 0023 (`docs/adr/0023-use-distinct-range-functions-and-two-cell-comments.md`) and the
      Comment paragraph of ADR 0033 (`docs/adr/0033-partition-a-row-by-parse.md:13`) each note
      that ADR 0035 supersedes them on this point. Do not rewrite their decisions.
- [x] `docs/adr/0019-map-orca-capabilities-onto-orcvs-language-families.md:38` still maps Comment
      to `#`. It was already stale against ADR 0023; correct it to `||`.
- [x] `.scratch/v1-release/definition-of-done.md:13` and
      `.scratch/v1-roadmap-wayfinding/issues/01-name-the-shipped-language-inventory.md:38` both
      state the `##` spelling as shipped fact. Update both when this lands, not before.

## Comments

2026-09-09: Raised by the release-candidate audit against `sequence-values/05`. Modelling the
Comment as a Function with a Grid-length parameter list was considered and rejected: operand
counts are fixed for each Function in `define_functions!`, `MAX_OPERANDS` derives from that table
at compile time and holds operands inline, operands are typed through `Token`, and a Function must
answer a value or an effect. A Cell claim carries none of those costs.

2026-09-09: Spelling changed from `##` to `||`, with the Comment declared the last member of the
operandless doubled-glyph family. `!!` and `//` were considered and refused. ADR 0035 records the
decision, the family rule it rests on, and the refused alternatives. The ticket now owns both the
respelling and the move into the parse, because the sites are the same sites.

2026-09-09: Design review found three property premises the original ticket did not name — the
two-Cell Span assertion, the complete-implies-Atoms `expect` on the `lang` side, and the
Atoms/diagnostic biconditional with its render-back check on the `orcvs` side. All three are added
above. The `lang`-side one fails as a panic rather than an assertion, so it will not read as a
property failure when it fires.
