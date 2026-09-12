# 02 — Decide what an unfilled operand Cell shows

**What to build:** A decision, recorded where the vocabulary lives, about whether an operand Cell a
Function declares but nothing fills shows a character or is painted by its declared type.

**Blocked by:** 01 — Carry the Token to the Render Frame and retire the Glyph classifier.

**Status:** needs-triage

- [ ] The question is answered for all of `Number`, `Note`, `Char`, `Atom` and `Sequence` operands,
      not only the two that `GlyphString` currently spells.
- [ ] Whichever way it goes, `GlyphString`'s remaining reason to exist is stated or the type is
      deleted.
- [ ] If operand slots are painted rather than spelled, `03`'s handover carries the declared-type
      colours; if they are spelled, the spelling table returns with a producer that reaches it.

## Comments

`GlyphString` spells `h` for an unfilled Number slot, `n` for a Note and `F` for a Function. That is
the same terminal inheritance `Marker` and `Highlight` were: standing a character in because a
terminal has only characters to stand anything in with. A console that paints can colour the empty
Cell in its declared type and spell nothing.

The reason this is a decision and not a consequence: those spellings read as deliberate
operand-placeholder design rather than leftovers — Orca shows operand hints, and an unfilled slot
you can see the shape of is genuinely useful while writing. Retiring them is a judgement about what
the console should look like, which is why `01` deletes the *dead* Glyph vocabulary and leaves this
alone.

Note that these spellings are reached today, which is what makes this a choice between keeping a
working feature and removing one. `LanguageMap` gives every Cell an Expression's claim covers that
claim's classification whether or not the Cell holds a byte — `language_map.rs:1584` states the rule
— so an Addition's reserved but unfilled operand Cells answer `Number` with no content and spell
`h`. `paint.rs:633` asserts exactly that. The producer this decision would need already exists:
keeping the placeholders costs nothing to build, and retiring them is a deletion with a visible
effect on screen, not the removal of dead vocabulary.
