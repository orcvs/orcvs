# 06 — Record the glossary authority rule

**What to build:** Add the authority rule to `AGENTS.md`, beside the existing verification rules.
When a property test and CONTEXT.md disagree, the glossary is correct.

**Status:** ready-for-agent

**Tags:** Improvement

- [ ] `AGENTS.md` states that CONTEXT.md and the ADRs are authoritative over a property test.
- [ ] It states that a wrong glossary sentence is corrected in CONTEXT.md in the same change.
- [ ] It states that a property is never weakened to match the code.
- [ ] It states that properties are written only for behaviour that exists.
- [ ] The wording matches the existing "Sources of truth" section rather than repeating it.

## Comments

This is a working practice, not an architecture decision, so it belongs in `AGENTS.md` and not in an
ADR. No ADR records a property-testing decision; ADR 0022, cited here before, is the core/UI crate
boundary, and mentions property tests only as one of the build costs the split removed.

The rule is what makes an executable specification worth having. Without it, a red property during
the language migration gets fixed the easy way, and the properties slowly drift into describing
whatever the code happens to do.

CONTEXT.md already hedges that it "does not claim that every term's complete behavior is implemented
yet". The rule and that hedge must not contradict each other: an unimplemented term has no property,
so it cannot disagree with one.

### Audit at cad296df — 2026-09-29

- The rule is still absent. `AGENTS.md` (which `CLAUDE.md` links to) has only "Architecture and
  vocabulary: `CONTEXT.md` and `docs/adr/`" (`AGENTS.md:7`) and no authority rule; nothing in
  `docs/` or `.agents/skills/` carries one either. The only statement of "written only for behaviour
  that exists" is in `.scratch/property-testing/spec.md:15`. The CONTEXT.md hedge quoted above is at
  `CONTEXT.md:3`. No criterion is met.
- The earlier comment said ADR 0022 covers this effort's one ADR-worthy decision. ADR 0022
  (`docs/adr/0022-keep-the-core-crate-free-of-the-ui-toolkit.md`) is the crate-boundary decision and
  names property tests only in passing (`:12`). Corrected in place.
