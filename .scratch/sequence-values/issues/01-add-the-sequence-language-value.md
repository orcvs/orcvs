# 01 — Add the Sequence language value

**What to build:** Represent one flat ordered Sequence of Atoms as a language value that can cross
Function evaluation without becoming Source writes prematurely.

**Blocked by:** `.scratch/orcvs-language-migration/issues/04-select-a-disjoint-note-encoding.md`.

**Status:** ready-for-agent

- [ ] Sequence preserves Atom order and each Atom's type.
- [ ] Atom promotion creates singleton Sequences where a Sequence Function requires it.
- [ ] Nested Sequences are impossible or flattened at the one controlled construction point.
- [ ] Empty and singleton Sequences have explicit behavior.
- [ ] Encoding is deterministic and concatenates complete Atom encodings.
- [ ] Language errors distinguish Atom, Sequence, and incompatible operands.

## Comments

Sequence is a value, not a Cell batch or implicit Tick Plan write list.
