# 14 — Bring ADR 0053 and the Theme record up to date

**What to build:** Make the decision record and `console/src/theme.md` state what `main` does after the 2026-09-22 retune, `08`'s validator and `04`'s colour-vision gate.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] *(Moved to `18`.)* ADR 0053 gains a dated amendment. The "Contrast acceptance preserves Okabe–Ito's Sequence colour… accepted exception" paragraph (around line 67) and the "Known dark failures awaiting acceptance" table (lines 69-86, with the opaque tints `#0E1D25`, `#26240B`) are superseded: the retune cleared every failure, and both accepted lists are empty. The amendment states that.
- [x] ADR 0053 records the colour-vision gate: `CONFUSION_FLOOR`, the simulation and CIEDE2000.
- [x] ADR 0053's Consequences (around lines 106-118) stop describing Theme pickers. Theme selection is by identity in `~/.orcvs/config.toml` (`theme.dark` / `theme.light`), and the console offers no in-app picker; the "become a Theme picker for each of the two slots" sentence is replaced to match.
- [x] *(Moved to `18`.)* `console/src/theme.md:706-710` stops claiming Okabe–Ito's Region states repeat plain's figures. `region.background` is `#FFFFFF2B`. The Region and Portal-over-role figures are recorded: Bang/Region 4.63, Comment/Region 4.97, Ordinary/Region 11.81, and Portal-over-role 6.18–7.77. Recompute them rather than copying these.
- [x] *(Moved to `18`.)* `theme.md:301` and `:363` ("Five of the seven") agree with the "What moved" table at `:286-292`: six of the seven hues moved, four darkened until the contrast floor held and two past it for colour vision, and Sequence did not move. `:363` says five were darkened until the floor held, which counts Sequence among them; `:301` counts five and then Sequence again.
- [x] ADR 0044's Status line stops opening with "superseded by ADR 0050". 0050 was rejected, and 0052 refines 0044.
- [x] *(Moved to `18`.)* The amendment is reviewed by a human before merge, since it changes a decision record.

## Comments

**2026-09-23 — opened by the audit of the merged pull requests against their issues.**

**2026-09-24 — rechecked against `origin/main` after #128-#130 merged.** Every line is still open. #130 added the chrome figures to `theme.md` but left the Region claim, now at `theme.md:706-710`; line references were refreshed.

**2026-09-24 — split.** The `theme.md` corrections and the ADR 0053 exception amendment moved to `18`, which joins `release/v1`. The remaining lines (recording the colour-vision gate in ADR 0053, the pickers wording, ADR 0044's Status line) stay out of the release as record accuracy.

### Audit at cad296df — 2026-09-29

The pickers line described a state that no longer exists. `SelectedThemes::listed` is gone, and
5f35edc9 moved Theme selection into `~/.orcvs/config.toml` (`console/src/config.rs:158-159`).
`no_menu_offers_a_setting` (`console/src/console/kittest_tests.rs:271`) asserts that no menu
offers a Theme. ADR 0053's Consequences still say `03` "prepares the Theme pickers" (`:107`) and
that the menu items "become a Theme picker for each of the two slots" (`:118`), so the line now
asks the ADR to state config-file selection instead.

The other two open lines were re-checked:

- ADR 0053 has no mention of `CONFUSION_FLOOR` or CIEDE2000. The gate is recorded only in
  `console/src/theme.md`.
- ADR 0044's Status line still opens with "superseded by ADR 0050".

`console/src/theme_registry.rs:18-20` also still says `SelectedThemes` "lists its Themes in the
View menu's pickers". That is code, outside this ticket's record-only scope, but it is the same
stale wording.

### Resolved — 2026-10-10

All three open lines are done on `chore/theming-14-adr-0053-record`. Records only; no code changed.

- ADR 0053 gains "Amendment, 2026-10-10: the built-ins are gated for colour vision". It records
  `contrast::colour_vision::distinguish`: the eight glyph channels it compares at their displayed
  colours, the Viénot, Brettel & Mollon (1999) simulation and its weaker tritan row, CIEDE2000 at
  unit weights, `CONFUSION_FLOOR` at 5.0 gating protanopia and deuteranopia with no exception
  list, tritanopia measured and pinned at 0.60 and 1.54 but not gated, and the test-only scope.
  Every figure is the one the tests pin.
- ADR 0053 gains "Amendment, 2026-10-10: Themes are selected in the config file", and its
  Consequences keep the picker wording with a superseded marker, following the 2026-10-01
  amendment's convention. The Status line names both amendments.
- ADR 0044's Status line opens "accepted", and says that ADR 0052 refines it, as ADR 0002's Status
  phrases an amending ADR. It keeps the history: 0050 superseded it, and 0052 superseded 0050.

`CONTEXT.md`'s **Theme** entry says settings choose a Theme by name and never mentions a picker, so
it was left alone. `console/src/theme_registry.rs:19-20` still says `SelectedThemes` lists its
Themes in the View menu's pickers. That is code, outside this ticket's scope.

**The two ADR 0053 amendments change a decision record and need human review before merge.**

**2026-10-10 — the pickers wording is kept, not replaced.** The user chose to keep the Consequences' picker sentences under a superseded marker rather than replace them as the pickers line says: the amendment states present behaviour and the marked text stays as the decision's record, as the 2026-10-01 amendment does. The `theme_registry.rs` module doc is corrected by `15`.
