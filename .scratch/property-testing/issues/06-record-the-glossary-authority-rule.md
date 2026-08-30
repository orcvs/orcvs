# 06 — Record the glossary authority rule

**What to build:** Add the authority rule to `AGENTS.md`, beside the existing verification rules.
When a property test and CONTEXT.md disagree, the glossary is correct.

**Status:** ready-for-agent

- [ ] `AGENTS.md` states that CONTEXT.md and the ADRs are authoritative over a property test.
- [ ] It states that a wrong glossary sentence is corrected in CONTEXT.md in the same change.
- [ ] It states that a property is never weakened to match the code.
- [ ] It states that properties are written only for behaviour that exists.
- [ ] The wording matches the existing "Sources of truth" section rather than repeating it.

## Comments

This is a working practice, not an architecture decision, so it belongs in `AGENTS.md` and not in an
ADR. ADR 0022 covers the one decision in this body of work that meets the ADR bar.

The rule is what makes an executable specification worth having. Without it, a red property during
the language migration gets fixed the easy way, and the properties slowly drift into describing
whatever the code happens to do.

CONTEXT.md already hedges that it "does not claim that every term's complete behavior is implemented
yet". The rule and that hedge must not contradict each other: an unimplemented term has no property,
so it cannot disagree with one.
