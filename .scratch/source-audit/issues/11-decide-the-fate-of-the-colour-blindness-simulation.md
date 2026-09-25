# 11 — Decide the fate of the colour-blindness simulation

**What to build:** A decision, then its implementation: either the colour-vision simulation and distinguishability checks become part of Theme contrast validation, or they leave shipped code. Today about 490 lines of `console/src/contrast.rs` ship uncalled, kept compiling by fourteen dead-code expectations, and the Theme by-key accessors are kept the same way.

**Blocked by:** None — can start immediately.

**Status:** ready-for-human

- [ ] The decision and its reason are recorded in this ticket.
- [ ] If kept: Theme validation calls the simulation and reports indistinguishable roles as a notice, with tests.
- [ ] If removed: the simulation lives only under test or in a separate tool, and no dead-code expectation remains in the contrast module.
- [ ] The Theme by-key accessors get the same treatment.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Still valid; `contrast.rs` is unchanged since the baseline. Context for the decision: since 8a5f9ef6 the web build loads no Theme documents, so Theme resolution and `ThemeDocument` are native-only — validation added here would run only natively.
