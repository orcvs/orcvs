# 14 — Bring ADR 0053 and the Theme record up to date

**What to build:** Make the decision record and `console/src/theme.md` state what `main` does after the 2026-09-22 retune, `08`'s validator and `04`'s colour-vision gate.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] ADR 0053 gains a dated amendment. The "Contrast acceptance preserves Okabe–Ito's Sequence colour… accepted exception" paragraph (around line 67) and the "Known dark failures awaiting acceptance" table (lines 69-83, with the opaque tints `#0E1D25`, `#26240B`) are superseded: the retune cleared every failure, and both accepted lists are empty. The amendment states that.
- [ ] ADR 0053 records the colour-vision gate: `CONFUSION_FLOOR`, the simulation and CIEDE2000.
- [ ] ADR 0053's Consequences say the pickers were built and exposed under `04` (`theme_selection::SelectedThemes::listed`, on `theming-04-switching`), not prepared by `03`.
- [ ] `console/src/theme.md:652-660` stops claiming Okabe–Ito's Region states repeat plain's figures. `region.background` is `#FFFFFF2B`. The Region and Portal-over-role figures are recorded: Bang/Region 4.63, Comment/Region 4.97, Ordinary/Region 11.81, and Portal-over-role 6.18–7.77. Recompute them rather than copying these.
- [ ] `theme.md:300` and `:361` ("Five of the seven") agree with the "What moved" table at `:286-292`: four bound by the floor, Sequence unmoved, two set by colour vision.
- [ ] ADR 0044's Status line stops opening with "superseded by ADR 0050". 0050 was rejected, and 0052 refines 0044.
- [ ] The amendment is reviewed by a human before merge, since it changes a decision record.

## Comments

**2026-09-23 — opened by the audit of the merged pull requests against their issues.** An untracked `docs/adr/0053-one-theme-styles-the-whole-console.md.bak` in the main checkout shares the number 0053. Delete it before running `scripts/check-tooling-contract.sh` locally.
