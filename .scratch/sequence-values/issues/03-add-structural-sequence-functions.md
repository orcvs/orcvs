# 03 — Add structural Sequence Functions

**What to build:** Implement the four structural Sequence Functions — Reverse `:<`, Concatenate
`:&`, Select `:?`, and Replace `:=` — with the exact contracts in ADR 0007.

**Blocked by:** 01 — Add the Sequence language value; orcvs-language-migration/01; lang-foundations/02; pre-split-defects/15 — Bound the Operand Stack by the Expression length.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Reverse changes Atom order but never reverses an Atom encoding.
- [ ] Concatenate promotes Atoms, stays flat, and treats empty Sequence as identity.
- [ ] Select uses a Number index modulo non-empty Sequence length.
- [ ] Replace returns a new same-length Sequence and permits a different replacement Atom type.
- [ ] Empty and invalid operands diagnose as ADR 0007 specifies.
- [ ] Every Function parses and round-trips through its canonical two-Cell spelling.
- [ ] Structural operations preserve a Bang member's type and encoding. (From issue 01, which
      covers Bang only through construction, promotion, and encoding.)
- [ ] All four stay generic over Atom type, because none of them reinterprets an Atom.

## Comments

### Scope correction, 2026-09-04

This issue originally also owned Range `:-`. It does not any more. ADR 0023 keeps the structural
Sequence Functions generic precisely because they do not reinterpret their Atoms, while the two
Range Functions are monomorphic and each fixes its own operand and result type. Carrying Range here
put a type-directed Function in the generic issue and left `:#` with no owner at all.

Range `:-` and `:#` are now owned together by `issues/05-add-the-range-functions.md`. The acceptance
line that read "Range handles ascending, descending, equal, Number, Note, and mixed-type bounds" is
not deleted, only moved: issue 05 carries the mixed-type diagnostic explicitly, because that is the
behaviour `orcvs-language-migration/04` settled and it must not be lost in the split.
