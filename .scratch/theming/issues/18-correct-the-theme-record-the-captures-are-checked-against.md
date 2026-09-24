# 18 — Correct the Theme record the captures are checked against

**What to build:** `console/src/theme.md` and ADR 0053 state what the candidate's Themes are, so the release captures are judged against a true record and the release carries no contrast exclusion it does not have. Moved from `14`.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] `theme.md` stops claiming Okabe–Ito's Cursor and Region fills are unset and so repeat `plain`'s figures. `region.background` is `#FFFFFF2B`. The Region and Portal-over-role figures are recomputed and recorded.
- [ ] `theme.md`'s "Five of the seven" statements agree with its "What moved" table: four bound by the floor, Sequence unmoved, two set by colour vision.
- [ ] `theme.md`'s header stops saying every value on the page belongs to Okabe–Ito; the page records both built-in Themes.
- [ ] ADR 0053 gains a dated amendment: the Sequence "accepted exception" paragraph and the "Known dark failures awaiting acceptance" table are superseded, because the 2026-09-22 retune cleared every failure and both accepted lists are empty.
- [ ] The ADR amendment is reviewed by a human before merge.

## Comments

**2026-09-24 — split from `14` by the release-membership audit.** `theme.md` is the record `restyle-egui-console/03`'s captures are checked against, and the ADR's exception table describes an exclusion the definition of done forbids unless recorded. Blocks `restyle-egui-console/03`.
