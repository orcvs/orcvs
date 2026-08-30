# 03 — Parser totality on ASCII input

**What to build:** Check that the parser terminates and never panics on arbitrary ASCII input, and
that its two entry points agree. Generate strings from the full printable ASCII range, not only from
valid Orcvs spellings.

**Blocked by:** 01 — Add proptest for native targets.

**Status:** ready-for-agent

- [ ] `Parser::parse` never panics for any ASCII input up to `EXP_LEN`.
- [ ] `try_parse` never panics, and returns an error wherever `parse` reports invalid.
- [ ] Input longer than `EXP_LEN` produces `ExpressionTooLong` rather than a panic or a truncation.
- [ ] A successful `try_parse` consumes the whole input, leaving no trailing content.
- [ ] Every `Atom` renders through `Display` and parses back to an equal `Atom`.
- [ ] The generator covers the space character and the `#` comment character.

## Comments

`AGENTS.md` names this exact obligation: "parser/protocol boundary: boundary or property tests". The
parser is the widest input surface in the workspace, because every keystroke reaches it.

The round trip is the strongest property here, but it only holds for Atoms whose canonical spelling
is settled. CONTEXT.md records that the Note encoding "remains to be selected", so exclude `Atom::Note`
from the round trip until `orcvs-language-migration/04` lands, and say so in a comment rather than
silently narrowing the generator.

Generate raw input. A generator that only produces valid Expressions tests the generator, not the
parser, and it slowly becomes a second implementation of the grammar.
