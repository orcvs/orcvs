# 03 — Parser totality on ASCII input

**What to build:** Check that strict Expression parsing and permissive Live Edit analysis terminate,
never panic, and preserve their distinct contracts on arbitrary ASCII input. Generate strings from
the full printable ASCII range, not only valid Orcvs spellings.

**Blocked by:** 01 — Add proptest for native targets; lang-foundations/08 — Separate Source analysis from strict parsing; language-map/02 — Derive Expressions, roots, and diagnostics.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Strict parsing never panics and returns only complete evaluable entries or a typed error.
- [ ] Permissive analysis never panics, preserves complete recognized units, and reports incomplete
      or invalid Source explicitly without placeholder runtime values.
- [ ] Input longer than `EXP_LEN` produces `ExpressionTooLong` rather than a panic or a truncation.
- [ ] A successful strict parse consumes the whole Expression, leaving no trailing content.
- [ ] Recovery advances at the documented Cell and every diagnostic refers to the same Language Map
      revision as its Position or Footprint.
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
