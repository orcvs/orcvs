# 08 — Validate a Theme’s composited contrast

**What to build:** A validator that measures a resolved Theme's text contrast in its actual painted states, replacing `style.rs`'s assertions about Okabe–Ito's exact values. Report the role, state, effective foreground and background, measured ratio, and applicable floor for each result.

**Blocked by:** 06 — Paint the Source from a named Theme.

**Status:** ready-for-human — the composited-state validator, its tests and `theme.md`'s record
are complete and every local gate passes. `contrast::tests::shipped_theme_gate` is deliberately
`#[ignore]`d rather than green: the three failures below are confirmed through shipped
composition, kept failing and visible, and await an explicit human decision (accept or retune)
before that gate can be un-ignored. See the `2026-09-22` "Pending dropped; Sequence has no
reachable failure" comment.

**Tags:** release/v1

- [x] Validation consumes a resolved Theme and evaluates the effective foreground against the actual composited background for each reachable painted text state, following `06`'s fixed composition rules. Raw colour pairs alone are insufficient.
- [x] Source coverage includes role backgrounds, Diagnostic and Output Portal channels, Cursor and Region fills and their reachable overlaps (including a Cursor or Region over an Output Portal). Console text is checked against its panel and input backgrounds. Report each result by role and state. `selection.background` has no production painting consumer yet (`.scratch/theming/issues/03`'s job) and so has no reachable painted state to check — verified by search, not assumed. The Cursor Effect's animated `area` field is out of scope; documented and tested as a deliberate boundary, not measured (see the module doc and `painted_background_is_unaffected_by_the_cursor_area_effect`).
- [x] Alpha compositing matches painting. A transparent fact foreground leaves the underlying glyph visible, so validate that effective glyph rather than treating the disabled fact channel as text. Resolve translucent backgrounds against their underlying surface before measuring contrast.
- [x] Fixtures include text that passes against the bare Grid background but fails against its painted tint or selected background, partial-alpha foreground/background combinations, and a transparent fact foreground exposing the underlying Token. Assert both the measured result and its reported role/state.
- [x] The report carries its floor, its scope description, and each result's acceptance annotation in the returned data (`ContrastReport::floor`/`::scope`, `ContrastResult::accepted`), not only in rustdoc.
- [x] The floor is stated once, with its source, rather than repeated at each call site.
- [x] A test iterates every shipped Theme and rejects any contrast failure without an explicitly recorded acceptance (`shipped_theme_gate`, `#[ignore]`d — see Status). Acceptance is keyed on role, state, and the exact measured colour pair, not on the label alone, so a retune cannot stay silently accepted (`accepted_failure_match_requires_the_same_colour_pair_not_only_role_and_state`). Okabe–Ito's accepted list is empty: Sequence, its only candidate, has no reachable failing state left once Pending roles are dropped (see the dated comment) — recorded as "nothing to except," not as the exception withdrawn.
- [x] Confirm the known invalid Number and invalid Note Diagnostic contrast failures listed below through shipped rendering, and keep them failing and visibly pending explicit acceptance (`pending_contrast_failures_are_confirmed_through_shipped_composition`). A third, Atom, was found by the same shipped composition and is listed alongside them. None is silently accepted; explicit human acceptance is still needed before the shipped-Theme gate can pass.
- [x] The validator reports every accepted failure with its measured ratio and floor and never hides an unrecorded one (`accepted_failure_match_requires_the_same_colour_pair_not_only_role_and_state`; `pending_contrast_failures_are_confirmed_through_shipped_composition`'s `!result.accepted` assertions). Okabe–Ito has no accepted failure today to show remaining visible, since Sequence's exception is empty — the mechanism is proven at the unit level instead.
- [x] The validator itself is tested against a Theme built to fail, so a passing run is not vacuous.
- [x] `style.rs`'s current general ordering assertions go: the per-colour table pinning each measured ratio, the relative-to-Comment exceptions, and `comment_reads_dimmer_than_every_colour_but_its_named_exceptions_which_all_clear_the_floor`. Those are facts about Okabe–Ito, not rules a Theme must satisfy.
- [x] The Okabe–Ito Theme's own measurements are recorded in `console/src/theme.md` as a property of that Theme, so the record `syntax-highlighting/01` and `07` built is kept rather than deleted.
- [x] A failing Theme is reported, never refused. `07` shows the report on load; that UI is `07`'s own scope, not built here — `validate` returns a plain `ContrastReport`, never a `Result`.

### Known dark failures awaiting acceptance

Three reachable states are known below-floor results, not accepted
exceptions: an opaque Diagnostic foreground overlaying the declared role
background, outside an Output Portal and without a Cursor/Region fill
replacing it.

| State | Foreground | Background | Calculated ratio | Floor | Status |
|---|---|---|---:|---:|---|
| Invalid Number operand | `#D55E00` | `#0E1D25` | 4.446944:1 | 4.5:1 | Explicit acceptance pending |
| Invalid Note operand | `#D55E00` | `#26240B` | 4.053689:1 | 4.5:1 | Explicit acceptance pending |
| Invalid Atom operand | `#D55E00` | `#252625` | 3.927542:1 | 4.5:1 | Explicit acceptance pending |

Confirmed through the real shipped composition in
`contrast::tests::pending_contrast_failures_are_confirmed_through_shipped_
composition` (`console/src/contrast.rs`), the one source of truth for these
figures — this table and `shipped_theme_gate`'s `#[ignore]` reason both point
back to that test rather than restating them a third time. Do not whitelist
these into `accepted_failures`, lower the floor, or retune the colours
without an explicit human decision recorded here. The table is not an
exhaustive claim: a newly discovered failure still requires review.

## Comments

**What is actually being given up.** The current assertions catch a real class of mistake — `syntax-highlighting/07` strengthened them precisely because the old rule let a Comment retuned brighter than Number keep the suite green. That protection does not survive contact with arbitrary schemes: Solarized, Gruvbox and Nord each order their accents differently, and an ordering assertion would fail every one. The validator keeps the measurement and drops the ordering, which is the part that was Okabe–Ito's rather than Orcvs's.

**Sequence is the worked example.** `#0072B2` measures 4.05:1 on `#000000` — below the 4.5:1 floor — and `syntax-highlighting/01` named it an exception in prose. Under a validator it is a reported failure of the shipped Okabe–Ito scheme, visible rather than excused. Deciding whether to retune that slot or keep the exception is a question for whoever accepts this, and it is a better question than the one the prose exception was avoiding.

**2026-09-21 — ADR 0053 widens what is validated.** A Theme now carries named keys as well as Token slots. `diagnostic.foreground` and `output_portal.foreground` colour glyphs on the Grid, so they are measured against `base00` like any Token. `text` and `text.muted` are measured against `panel.background`. Sequence moved to `base0F`, whose colour varies most between published schemes. In Nord it is a blue close to Function's `base0D`, which is exactly the kind of finding the report exists to surface.

**2026-09-21 — Sequence exception confirmed by the user.** Preserve Okabe–Ito's existing Sequence colour as an explicit, reported exception. Keep the 4.5:1 floor and the measured failure visible. Tests reject additional unrecorded failures. This settles the retune-or-preserve question above and replaces the contradictory requirement that every shipped scheme pass without exception.

**2026-09-21 — composited text contrast confirmed.** The user confirmed validation against actual painted backgrounds, including Token tints, selection, Cursor and Region fills, and panel/input backgrounds for console text. Report the failing role and state. Colour distinguishability is a separate assessment: the earlier Nord example does not mean a text/background contrast ratio measures similarity between Sequence and Function colours. The older base00-only signature and acceptance are superseded above.

**2026-09-22 — one Orcvs format confirmed.** The user chose one versioned Orcvs Theme format with named style properties. Base16 is inspiration only; importing/conversion is deferred. This supersedes the earlier sixteen-slot, palette/template and published-scheme requirements. Native startup loading, web file imports, exact inheritance, strict validation and the confirmed appearance controls remain in scope.

**2026-09-22 — explicit role backgrounds confirmed.** Role backgrounds are explicit colour properties, such as `source.function.background`, `source.number.background`, `diagnostic.background` and `output_portal.background`. They composite over the uniform `cell.background` under the confirmed fixed precedence. Changing a role foreground does not recalculate its background. There is no `fill_tint` property or shared tint-strength setting in the Orcvs Theme format. Extract and verify the built-in role background colours from current rendering so its appearance is preserved; illustrative colours in the interview are not accepted defaults.

**2026-09-22 — implemented as a composited-state validator; superseded an earlier `base00`-only one.** `console/src/contrast.rs` measures every reachable painted text state's effective foreground against its effective background, reusing `style::cell_visuals_with_cursor_colour` and the extracted `style::cell_background` — the same functions `Paint::derive_with_theme` calls per Cell, so painting and validation cannot independently drift. `ContrastReport` (from `validate(theme: &Theme)`) carries `floor`, `scope`, and a `Vec<ContrastResult>`; each result carries `role`, `state`, effective `foreground`/`background`, `ratio`, and whether it is an accepted exception. `Role` and `State`/`CursorPlacement` are small enums with `Display` impls, not raw strings. `Placement`'s former `OutputPortal` variant is gone: Output Portal is now an independent `bool` crossed with every `CursorPlacement`, since the Cursor or a Region can genuinely sit on a Cell an Output Portal Reservation also covers. `selection.background` was checked and has no production painting consumer yet, so it is out of scope until `03` wires one. The Cursor Effect's animated `area` field is a documented, tested exclusion (see the module doc and `painted_background_is_unaffected_by_the_cursor_area_effect`): its on-screen alpha depends on Glitch settings and continuous animation, neither part of the Theme this validator measures. `console.rs`'s input background is painted inside a panel, so the input check composites over the panel layer, not over the window backdrop directly. `console/src/theme.md` records the real composited `plain`-state figures — Function, for one, is 5.35:1 against its own tint, not 6.14:1 against bare black — and states plainly that the Cursor placement always replaces a role's background outright (`style::cell_visuals_with_cursor_colour`'s "the Cursor's own fill wins outright"), unlike Region, which only replaces a *transparent* one.

**2026-09-22 — Pending dropped; Sequence has no reachable failure.** A Pending Cell draws no glyph (`paint::tests::every_pending_operand_token_draws_no_glyph_through_the_real_paint_path` proves this through the real paint path), so there is no foreground for a Pending state to measure; `Role` carries no Pending variant. Consequence: Sequence's only other role, `Sequence, Invalid`, passes the floor on its own (`diagnostic.foreground` replaces Sequence's colour outright once Invalid), so Sequence has no reachable failing state left — `accepted_failures` for `okabe-ito` is empty. This is recorded as "nothing left to except," not as the exception withdrawn: `#0072B2` and the 4.5:1 floor are unchanged. Acceptance is now keyed on role, state, *and* the exact measured colour pair (`AcceptedFailure::matches`), not the label alone, so a future retune cannot stay silently accepted. The remaining pending-acceptance failures are the invalid Number/Note/Atom operand Diagnostic states only, confirmed in `contrast::tests::pending_contrast_failures_are_confirmed_through_shipped_composition` — the one source of truth for their ratios.
