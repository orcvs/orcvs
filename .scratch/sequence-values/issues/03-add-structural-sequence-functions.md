# 03 — Add structural Sequence Functions

**What to build:** Implement Range `:-`, Reverse `:<`, Concatenate `:&`, Select `:?`, and Replace
`:=` with the exact contracts in ADR 0007.

**Blocked by:** 01 — Add the Sequence language value; orcvs-language-migration/01; lang-foundations/02.

**Status:** ready-for-agent

- [ ] Range handles ascending, descending, equal, Number, Note, and mixed-type bounds.
- [ ] Reverse changes Atom order but never reverses an Atom encoding.
- [ ] Concatenate promotes Atoms, stays flat, and treats empty Sequence as identity.
- [ ] Select uses a Number index modulo non-empty Sequence length.
- [ ] Replace returns a new same-length Sequence and permits a different replacement Atom type.
- [ ] Empty and invalid operands diagnose as ADR 0007 specifies.
- [ ] Every Function parses and round-trips through its canonical two-Cell spelling.
