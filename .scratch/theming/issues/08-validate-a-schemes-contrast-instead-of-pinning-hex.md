# 08 — Validate a Theme’s composited contrast

**What to build:** A validator that measures a resolved Theme's text contrast in its actual painted states, replacing `style.rs`'s assertions about Okabe–Ito's exact values. Report the role, state, effective foreground and background, measured ratio, and applicable floor for each result.

**Blocked by:** 06 — Paint the Source from a named Theme.

**Status:** ready-for-human — the validator, its tests and `theme.md`'s record are complete and
every local gate passes; Sequence's retune-vs-exception decision, named below and in this file's
`2026-09-22` comment, is for a human to accept.

**Tags:** release/v1

- [ ] Validation consumes a resolved Theme and evaluates the effective foreground against the actual composited background for each reachable painted text state, following `06`'s fixed composition rules. Raw colour pairs alone are insufficient.
- [ ] Source coverage includes role backgrounds, Diagnostic and Output Portal channels, selection, Cursor and Region fills and their reachable overlaps. Console text is checked against its panel and input backgrounds. Report each result by role and state; do not substitute impossible cross-products of unrelated colours for actual painted states.
- [ ] Alpha compositing matches painting. A transparent fact foreground leaves the underlying glyph visible, so validate that effective glyph rather than treating the disabled fact channel as text. Resolve translucent backgrounds against their underlying surface before measuring contrast.
- [ ] Fixtures include text that passes against the bare Grid background but fails against its painted tint or selected background, partial-alpha foreground/background combinations, and a transparent fact foreground exposing the underlying Token. Assert both the measured result and its reported role/state.
- [ ] The report describes its text-contrast scope. It does not claim to measure pairwise Token-colour distinguishability, colour-vision accessibility, or focus/border visibility; those require separate assessment.
- [ ] The floor is stated once, with its source, rather than repeated at each call site.
- [ ] A test iterates every shipped Theme and rejects any contrast failure without an explicitly recorded acceptance. The accepted exception is Okabe–Ito's Sequence (`source.sequence`, `#0072B2` against the bare Grid background, `#000000`, approximately 4.05:1). Preserve that colour and the 4.5:1 floor; this exception does not exempt other roles, Themes or newly measured failing states. Record additional failures discovered by the expanded state coverage for explicit review rather than silently accepting them.
- [ ] Confirm the known invalid Number and invalid Note Diagnostic contrast failures listed below through shipped rendering. Keep them failing and visibly pending explicit acceptance; preserving the dark colours does not itself authorise new exceptions. Obtain explicit acceptance before the shipped-Theme gate can pass with these colours.
- [ ] The validator reports the accepted Sequence failure with its measured ratio and floor. Acceptance may annotate the report, but never hides the failure or converts it to a passing measurement. Test that the accepted failure remains visible and an additional unrecorded failure fails the shipped-Theme gate.
- [ ] The validator itself is tested against a Theme built to fail, so a passing run is not vacuous.
- [ ] `style.rs`'s current general ordering assertions go: the per-colour table pinning each measured ratio, the relative-to-Comment exceptions, and `comment_reads_dimmer_than_every_colour_but_its_named_exceptions_which_all_clear_the_floor`. Those are facts about Okabe–Ito, not rules a Theme must satisfy. Sequence's accepted below-floor result moves into the explicit shipped-Theme acceptance test above.
- [ ] The Okabe–Ito Theme's own measurements are recorded in `console/src/theme.md` as a property of that Theme, so the record `syntax-highlighting/01` and `07` built is kept rather than deleted.
- [ ] A failing Theme is reported, never refused. `07` shows the report on load.

### Known dark failures awaiting acceptance

The following reachable invalid-operand states are known below-floor results,
not accepted exceptions. Opaque Diagnostic foreground overlays the declared role
background outside an Output Portal and without a Cursor/Region fill replacing it.

| State | Foreground | Background | Calculated ratio | Floor | Status |
|---|---|---|---:|---:|---|
| Invalid Number operand | `#D55E00` | `#0E1D25` | 4.446944:1 | 4.5:1 | Explicit acceptance pending |
| Invalid Note operand | `#D55E00` | `#26240B` | 4.053689:1 | 4.5:1 | Explicit acceptance pending |

Recorded 2026-09-22 from the specified opaque colours using the standard sRGB
relative-luminance ratio. Confirm both through shipped rendering/composition in
the implementation. Exact dark appearance preservation cannot satisfy the shipped
Theme acceptance test until these failures are explicitly accepted; retain the
failures and do not silently whitelist them, lower the floor or retune colours.
The existing Sequence exception remains the only accepted exception. The table
is not an exhaustive claim: newly discovered failures still require review.

## Comments

**What is actually being given up.** The current assertions catch a real class of mistake — `syntax-highlighting/07` strengthened them precisely because the old rule let a Comment retuned brighter than Number keep the suite green. That protection does not survive contact with arbitrary schemes: Solarized, Gruvbox and Nord each order their accents differently, and an ordering assertion would fail every one. The validator keeps the measurement and drops the ordering, which is the part that was Okabe–Ito's rather than Orcvs's.

**Sequence is the worked example.** `#0072B2` measures 4.05:1 on `#000000` — below the 4.5:1 floor — and `syntax-highlighting/01` named it an exception in prose. Under a validator it is a reported failure of the shipped Okabe–Ito scheme, visible rather than excused. Deciding whether to retune that slot or keep the exception is a question for whoever accepts this, and it is a better question than the one the prose exception was avoiding.

**2026-09-21 — ADR 0053 widens what is validated.** A Theme now carries named keys as well as Token slots. `diagnostic.foreground` and `output_portal.foreground` colour glyphs on the Grid, so they are measured against `base00` like any Token. `text` and `text.muted` are measured against `panel.background`. Sequence moved to `base0F`, whose colour varies most between published schemes. In Nord it is a blue close to Function's `base0D`, which is exactly the kind of finding the report exists to surface.

**2026-09-21 — Sequence exception confirmed by the user.** Preserve Okabe–Ito's existing Sequence colour as an explicit, reported exception. Keep the 4.5:1 floor and the measured failure visible. Tests reject additional unrecorded failures. This settles the retune-or-preserve question above and replaces the contradictory requirement that every shipped scheme pass without exception.

**2026-09-21 — composited text contrast confirmed.** The user confirmed validation against actual painted backgrounds, including Token tints, selection, Cursor and Region fills, and panel/input backgrounds for console text. Report the failing role and state. Colour distinguishability is a separate assessment: the earlier Nord example does not mean a text/background contrast ratio measures similarity between Sequence and Function colours. The older base00-only signature and acceptance are superseded above.

**2026-09-22 — one Orcvs format confirmed.** The user chose one versioned Orcvs Theme format with named style properties. Base16 is inspiration only; importing/conversion is deferred. This supersedes the earlier sixteen-slot, palette/template and published-scheme requirements. Native startup loading, web file imports, exact inheritance, strict validation and the confirmed appearance controls remain in scope.

**2026-09-22 — explicit role backgrounds confirmed.** Role backgrounds are explicit colour properties, such as `source.function.background`, `source.number.background`, `diagnostic.background` and `output_portal.background`. They composite over the uniform `cell.background` under the confirmed fixed precedence. Changing a role foreground does not recalculate its background. There is no `fill_tint` property or shared tint-strength setting in the Orcvs Theme format. Extract and verify the built-in role background colours from current rendering so its appearance is preserved; illustrative colours in the interview are not accepted defaults.

**2026-09-22 — prior `base00`-only implementation, and its code-review round, superseded.** The signature and acceptance work recorded on this branch before the rebase onto `theme-paint-source` (a `validate(theme: &Theme)` measuring only the seven Tokens plus `diagnostic.foreground`/`output_portal.foreground` against `base00`, and `text`/`text.muted` against `panel.background`, plus a subsequent code-review round that moved it into a `contrast` submodule, replaced Okabe-Ito-pinning tests with synthetic ones, and reworked the shipped-Theme gate) answered ADR 0053's earlier, narrower scope. It is superseded by the composited-state validator this issue now describes, not merely extended: the earlier report shape and every one of its tests (`every_shipped_theme_clears_the_floor_except_sequences_recorded_exception`, `every_shipped_theme_matches_its_accepted_failure_record`, and their predecessors) are gone with it, replaced below by the reimplementation this rescoped issue asks for. The signature deviation this issue's earlier comments discussed (`scheme`/`template` having no referent once `06` resolved straight to a `Theme`) still holds under the new named-property `Theme` format, so `validate` continues to take a resolved `&Theme`. The `contrast` submodule shape, the `ContrastResult::passes()` method, and the single scoped `expect(dead_code)` attribute the review settled on are carried forward into the reimplementation below, since those were about code organisation rather than about the superseded `base00`-only scope.
