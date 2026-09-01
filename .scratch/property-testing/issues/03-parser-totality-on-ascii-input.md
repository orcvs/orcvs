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
- [ ] The generator covers the space character, incomplete `#`, and the `##` Comment introducer.

## Comments

`AGENTS.md` names this exact obligation: "parser/protocol boundary: boundary or property tests". The
parser is the widest input surface in the workspace, because every keystroke reaches it.

The round trip is contextual rather than a standalone parse law for Operand Literals. Cover every
Note value from `C/` through `G9` inside a Function's Note operand position and every Number value
inside a Number operand position. Overlapping characters such as `C4` must round-trip according to
that fixed signature; a standalone operand literal is invalid.

Generate raw input. A generator that only produces valid Expressions tests the generator, not the
parser, and it slowly becomes a second implementation of the grammar.
