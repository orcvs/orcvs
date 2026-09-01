# 01 — Add the Sequence language value

**What to build:** Represent one flat ordered Sequence of Atoms as a language value that can cross
Function evaluation without becoming Source writes prematurely.

**Blocked by:** orcvs-language-migration/04 — Confirm contextual Number and Note literals.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Sequence preserves Atom order and each Atom's type.
- [ ] Atom promotion creates singleton Sequences where a Sequence Function requires it.
- [ ] Nested Sequences are impossible or flattened at the one controlled construction point.
- [ ] Empty and singleton Sequences have explicit behavior.
- [ ] Encoding is deterministic and concatenates complete Atom encodings.
- [ ] Bang is a permitted Atom and structural Sequence operations preserve its type and encoding.
- [ ] Activation Characters gain no Expression-operand or Sequence behavior unless the focused
      Activation prototype explicitly selects and specifies that model.
- [ ] Language errors distinguish Atom, Sequence, and incompatible operands.

## Comments

Sequence is a value, not a Cell batch or implicit Tick Plan write list.
