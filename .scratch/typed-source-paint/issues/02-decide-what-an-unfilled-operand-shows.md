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

Note that nothing reaches these spellings today either. `LanguageMap` assigns a classification only
to a Cell holding a parsed token or a non-space byte, so a Cell with no content carries none, and
the blank path is unreachable rather than merely unused. Deciding to keep the placeholders therefore
means building the producer as well — the Render Frame would have to classify the empty Cells a
Function's arity claims, which it does not do now.
