# 06: Decide and build optional Chord Play roots

**What to build:** Decide what an empty pattern row means for Raw Chord Play `!:` and Timed Chord Play `!#`, then build it. Declaring `root: Optional<Note>` is not enough on its own, because an empty row also blanks the Interval Set, voicing, inversion and, on `!#`, length, and each of those still diagnoses. ADR 0104 let a blank root excuse them, which the type model cannot express per operand. See `../spec.md`, "Out of Scope".

**Blocked by:** 02

**Status:** needs-triage

- [ ] A decision is recorded: either those operands are optional too and the body plays nothing without a root, or the row stays diagnosed when it is blank.
- [ ] If they become optional, a blank root with a present set still plays nothing, and a present root with a blank set is still diagnosed.
- [ ] Behaviour is tested through `Source::execute` on the written, Portal and nested routes.
- [ ] #371's driven-chord tests are brought across with updated expectations.
