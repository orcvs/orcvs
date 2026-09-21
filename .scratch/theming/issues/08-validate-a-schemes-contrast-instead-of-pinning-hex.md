# 08 — Validate a scheme's contrast instead of pinning hex

**What to build:** A validator that measures any scheme against the template and reports each Token's contrast, replacing `style.rs`'s assertions about Okabe–Ito's exact values.

**Blocked by:** 06 — Paint the Source from a scheme.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] `validate(scheme, template)` answers each Token's measured WCAG ratio against the scheme's `base00`, and which fail a stated floor.
- [ ] The floor is stated once, with its source, rather than repeated at each call site.
- [ ] Every shipped scheme passes, asserted by a test that iterates them — so adding a scheme cannot quietly ship one that fails.
- [ ] The validator itself is tested against a scheme built to fail, so a passing run is not vacuous.
- [ ] `style.rs`'s current assertions go: the per-colour table pinning each measured ratio, the named exceptions (Sequence below the floor; Bang, Function and Diagnostic below Comment), and `comment_reads_dimmer_than_every_colour_but_its_named_exceptions_which_all_clear_the_floor`. Those are facts about Okabe–Ito, not rules a scheme must satisfy.
- [ ] The Okabe–Ito scheme's own measurements are recorded in `console/src/theme.md` as a property of that scheme, so the record `syntax-highlighting/01` and `07` built is kept rather than deleted.
- [ ] A failing scheme is reported, never refused. `07` shows the report on load.

## Comments

**What is actually being given up.** The current assertions catch a real class of mistake — `syntax-highlighting/07` strengthened them precisely because the old rule let a Comment retuned brighter than Number keep the suite green. That protection does not survive contact with arbitrary schemes: Solarized, Gruvbox and Nord each order their accents differently, and an ordering assertion would fail every one. The validator keeps the measurement and drops the ordering, which is the part that was Okabe–Ito's rather than Orcvs's.

**Sequence is the worked example.** `#0072B2` measures 4.05:1 on `#000000` — below the 4.5:1 floor — and `syntax-highlighting/01` named it an exception in prose. Under a validator it is a reported failure of the shipped Okabe–Ito scheme, visible rather than excused. Deciding whether to retune that slot or keep the exception is a question for whoever accepts this, and it is a better question than the one the prose exception was avoiding.
