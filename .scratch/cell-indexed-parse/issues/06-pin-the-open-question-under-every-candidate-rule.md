# 06: Pin the open question under every candidate rule

**What to build:** Whether an operand slot the Parser never reached is claimed becomes one flag at
one place — is that Cell stamped, or left empty — and the failure that the question concerns is
tested under every candidate answer before the answer is chosen.

There are three candidates, not the two the open question lists. The commit the question is about
filtered declared slots with a comparison that deliberately kept the *first* missing slot, so that a
write abutting the run could complete the Function. That is neither "claim every declared Cell" nor
"claim only Cells that hold characters", and it needs a name so the behaviour is not reconstructible
only from a comparison operator.

This ticket does not answer the question. It makes every answer expressible and leaves the failure
guarded whichever answer wins.

**Blocked by:** 02

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] All three rules are named, and there is no default: every construction names one.
- [ ] Switching rules is one change at one place, and no consumer changes with it.
- [ ] A half-typed terminal Function with a result written into its unreached operand Cells is
      tested under each rule, and the outcome is stated for each.
- [ ] The failure the question concerns cannot recur silently under any rule.
- [ ] The third rule is recorded on `language-map/09` so whoever answers it has the complete set.

## Comments

`language-map/09` remains the place the decision is taken, and stays `ready-for-human`.
