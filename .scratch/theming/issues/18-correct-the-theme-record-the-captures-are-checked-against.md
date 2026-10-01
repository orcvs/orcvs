# 18 — Correct the Theme record the captures are checked against

**What to build:** `console/src/theme.md` and ADR 0053 state what the candidate's Themes are, so the release captures are judged against a true record and the release carries no contrast exclusion it does not have. Moved from `14`.

**Blocked by:** None — can start immediately.

**Status:** resolved

**Tags:** release/v1

- [x] `theme.md` stops claiming Okabe–Ito's Cursor and Region fills are unset and so repeat `plain`'s figures. `region.background` is `#FFFFFF2B`. The Region and Portal-over-role figures are recomputed and recorded.
- [x] `theme.md`'s "Five of the seven" statements agree with its "What moved" table: four bound by the floor, Sequence unmoved, two set by colour vision.
- [x] `theme.md`'s header stops saying every value on the page belongs to Okabe–Ito; the page records both built-in Themes.
- [x] ADR 0053 gains a dated amendment: the Sequence "accepted exception" paragraph and the "Known dark failures awaiting acceptance" table are superseded, because the 2026-09-22 retune cleared every failure and both accepted lists are empty.
- [x] The ADR amendment is reviewed by a human before merge.

## Comments

**2026-09-24 — split from `14` by the release-membership audit.** `theme.md` is the record `restyle-egui-console/03`'s captures are checked against, and the ADR's exception table describes an exclusion the definition of done forbids unless recorded. Blocks `restyle-egui-console/03`.

**2026-09-29 — audit at `cad296df`.** Every line still holds. The unset-fills claim is now at `console/src/theme.md:731-733` ("Okabe–Ito's are unset, so its own Cursor/Region states happen to repeat `plain`'s figures"), while `region_background` is `straight_rgba(0xFF_FF_FF_2B)` (`console/src/theme.rs:815`). "Five of the seven" is at `theme.md:300` and `:362`; the header claim is at `theme.md:7`. ADR 0053 still carries the Sequence "accepted exception" paragraph (`docs/adr/0053-one-theme-styles-the-whole-console.md:67`) and the "Known dark failures awaiting acceptance" table (`:69`).

**2026-10-01 — criteria 1–4 done on `docs/theme-record-correction`; the ADR amendment awaits human review.** Status is now `ready-for-human`: the one criterion left is a human review of the ADR 0053 amendment. The ticket resolves when the pull request merges after that review.

- Criterion 1: the Cursor/Region paragraph at the end of `theme.md`'s Source colours section now says `cursor.background` and `region.cursor.background` are none while `region.background` is `#FFFFFF2B`, and states what each does to Okabe–Ito's figures. Neither state repeats `plain` throughout: a Region Cell over a transparent role is measured on `#2B2B2B` (Ordinary 11.81:1, Comment 4.97:1, Bang 4.63:1), and the Cursor's own Cell drops a tinted role to bare `#000000` (Function 6.14:1, Number 9.10:1, Note 15.88:1, Invalid 5.43:1, Output Portal 9.32:1). Portal over a role: Number 7.30:1 on `#1F2016`, Note 6.76:1 on `#2D2506`, Atom 6.66:1 on `#2D2615`, Sequence 7.77:1 on `#171B10`, Bang 6.18:1 on `#171000`. The figures were not computed by hand. A throwaway test in `contrast::tests` printed every `contrast::validate` result for both built-ins, and was then removed. All 95 Okabe–Ito states clear the floor; the lowest is Atom, Invalid at 4.59:1.
- Criterion 2: both statements now say four (Number, Note, Function, Bang) were bound by the contrast floor, Sequence was unmoved, and Diagnostic and Output Portal were set by colour vision. The "What moved" table gives the same split.
- Criterion 3: the header now says the page records both built-ins, and that a value is Okabe–Ito's unless it sits under or names Orcvs Light.
- Criterion 4: ADR 0053 gains "Amendment, 2026-10-01: no shipped Theme carries a contrast exception", and the Status line points to it. Both superseded passages carry a superseded marker and are otherwise kept. Before it was written, the claim was checked: `contrast::accepted_failures` returns `&[]` for `okabe-ito` and `orcvs-light`, and `cargo nextest run --package console --locked -E 'test(/contrast::/)'` passes 35/35, including `shipped_theme_gate`, `sequence_has_no_reachable_failing_state` and `orcvs_light_has_nothing_to_except`. One nuance is recorded in the amendment: the retune is what cleared the two invalid-operand failures, but Sequence's exception lapsed for a different reason. No reachable state draws a Sequence glyph, so `#0072B2` on `#000000` is never measured.
