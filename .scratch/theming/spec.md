# Theme the console

**Goal:** Every part of the console's presentation is themeable, and one Theme styles all of it: the Source Grid and the console around it, from one clearly identified, versioned Orcvs format.

## Why

`restyle-egui-console` decided the console's look and finished: `01` squared the Source Grid viewport and `02` pinned the palette tokens `console/src/theme.md` records. What stayed open under that slug was not restyling but theming — selecting a chrome palette by theme, deciding a light one, keeping a restored preference, and the Source colours themselves. Those moved here; `restyle-egui-console/03` stayed behind because it is candidate visual verification `v1-release/01` depends on, and it photographs the theme rather than deciding it.

`syntax-highlighting/01` shipped `Theme → Source colours` — ten resolved colours in a settings value, persisted in full on every autosave — and deferred reconciling it with the palette. ADR 0051 was that reconciliation, and ADR 0053 corrects it: ADR 0051 conceded chrome to egui's dark/light `Visuals`, which left most of the console outside any theme. The requirement was always that all presentation is themeable.

The implementation contract is [schema.md](schema.md), including all named
properties, dark defaults, parser rules and examples. The
[implementation handoff](implementation-handoff.md) records delivery ownership
and verification. The earlier schema draft is retained only as a link redirect.

## Rules

**A Cell carries facts. The console draws them through channels. A theme is the mapping.** Facts are the parser's claim, whether a slot is written, whether a Cell is in an Output Portal Reservation, whether it is selected. Channels are glyph colour, fill, border, overlay. Which channel shows which fact is a theme decision, and `style.rs` stops making it in control flow.

**One versioned Orcvs format.** A Theme document declares `format = "orcvs-theme"`, `version = 1`, a display `name`, one built-in `inherits` identity, and named properties under `style`. Source glyphs, fills, borders and chrome all use named properties. Built-ins define complete values; custom Themes inherit omitted properties. No palette slots or slot template are required.

Planned Theme controls are colours, opacity, Grid and Cell colours, borders and
border widths. Grid background and Cell fills expose colour and opacity. Cell grid lines and Sector Seams each expose independent colour, opacity and width. Cursor, Region and Diagnostic borders expose colour, opacity and width. Widths are bounded; transparent colours can hide lines. Grid line and border widths are measured in display points and retain the same visible thickness as Grid zoom changes; they do not scale with Cell size. Existing square Cells, zoom and spacing remain unchanged.
Width bounds and stroke defaults follow ADR 0053. Preserve Grid structure, existing typography and layout, input
behaviour and separate motion settings. Painting and hit-testing must agree.

Grid/Cell border and Sector Seam widths accept finite values from 0 to 1 display point inclusive; chrome border widths accept 0 to 2 points inclusive. Width zero hides the stroke. Preserve normal-zoom defaults: Cell grid lines 0.5 points, Sector Seams 0.75 points, stationary Cursor/Region effect outlines 1 point, and existing visible chrome borders 1 point (absent borders remain absent). Fixed display-point widths intentionally replace the previous zoom-scaled stroke behaviour.

Cursor/Region effect width is nominal: retain the existing animated fragment variation of 0.45–1.25 times that width, without Grid zoom scaling. The 1-point maximum constrains nominal width, so an animated fragment can reach 1.25 points. Width zero hides every affected stroke; it does not implicitly disable separately coloured fills or change motion preferences.

Transparency reveals the underlying console surface; the application window remains opaque. Desktop/window transparency is outside this effort. Painting and contrast validation must use the same composited backgrounds.

The window backdrop and Grid background have separate Theme colours. The window backdrop must be opaque; panel, Grid and Cell layers above it may use alpha. A transparent Grid reveals the underlying console surface. Built-ins explicitly define both backgrounds; custom documents inherit them independently. Contrast validation uses the actual composite down to the opaque backdrop.

Optional Cursor fills distinguish omission, none and explicit transparent colour. Omission inherits the parent value. None removes the optional fill and uses the existing fallback: for the Cursor inside a Region, it falls back to the ordinary Cursor fill; for the ordinary Cursor, it supplies no Cursor fill. An explicit transparent colour remains a supplied value and does not trigger that fallback. These optional states apply only to `cursor.background` and `region.cursor.background`, not to every colour key.

`cell.background` is a single Theme-wide base fill for all Cells, transparent by default. It composites over the Grid background and beneath role backgrounds, Diagnostic/Output Portal channels and Cursor/Region fills. It does not count as a fact fill when evaluating Region fallback, so setting it cannot suppress Region highlighting. Existing precedence among those highlighting channels remains unchanged. Individual Cells do not store styling.

Role backgrounds are explicit colour properties, such as `source.function.background`, `source.number.background`, `diagnostic.background` and `output_portal.background`. They composite over the uniform `cell.background` under the confirmed fixed precedence. Changing a role foreground does not recalculate its background. There is no `fill_tint` property or shared tint-strength setting in the Orcvs Theme format. Extract and verify the built-in role background colours from current rendering so its appearance is preserved; illustrative colours in the interview are not accepted defaults.

**Base16 import is deferred.** Base16 can inspire built-in colours. A future converter would produce Orcvs Theme documents; native Base16 loading, sixteen-slot tables and extra published-scheme built-ins are outside this effort.

**Themes do not reorder facts.** Colours and channels are Theme values; painting precedence is fixed. Preserve the existing Function and Bang exceptions inside Output Portals, portal precedence over other claims, Cursor fill precedence and Region fill fallback. ADR 0053 records those rules; `06` documents and tests the overlap cases, including alpha compositing.

**Settings name Themes; Themes are documents; there are no overrides.** Settings hold a dark Theme, a light Theme, and a mode (follow the OS, or hold one). A different look is a custom Theme that inherits from one built-in Theme and lists what it changes. Nothing stores a resolved colour the viewer did not choose, so an improved built-in Theme reaches every install. This is the property `SourcePaintSettings` lacks and the reason it is replaced rather than migrated.

**Inheritance preserves parent values.** A custom Theme inherits its parent's resolved named properties and replaces only properties explicitly supplied by the document. Omitted properties remain unchanged; no palette-slot recalculation exists.

**Custom appearance follows the parent.** A custom Theme inherits its built-in parent's dark/light appearance. An omitted appearance resolves from the parent; an explicit conflicting declaration rejects the document with an error identifying the mismatch. A light custom Theme must start from a light built-in, and a dark custom Theme from a dark built-in.

**A Theme decides how things look, never how much they move.** Glitch amount and frequency are settings. egui's `Visuals` are built from the resolved Theme; they are an output of it, never a second authority.

Amount zero gives a clear, stationary frame; frequency zero freezes the current
effect and may retain fragmentation. Neither schedules cursor-effect repaints.
Reduced motion gives a clear, stationary frame without rewriting stored motion
preferences. Playback, Run Clock and input-driven repaints remain independent.

**Native files are authoritative.** Orcvs scans `~/.orcvs/themes/` and reads custom Theme files at startup, so external changes take effect only on the next launch for this release. There is no file watcher or reload action. This is the only native Theme directory; settings select Themes by filename stem, while a document's name is a display label. Built-in identities are reserved and cannot be replaced by files. Orcvs does not restore file values from cached application storage. Web imports use files, with the filename stem as identity and the document name as a display label. Reimporting the same identity updates its document after successful validation. Imported documents remain in browser storage when persistence is enabled. Settings hold Theme references on both targets. Web import is disabled for v1 and the web has the built-ins alone (`.scratch/menu-structure/issues/04`); the web rules here are the design for when it returns.

**Duplicate native identities have no winner.** If multiple native files have the same filename stem, all files with that identity are refused and the conflict is reported with the conflicting filenames. Directory enumeration order never chooses a winner. If the selected identity is conflicted, use the default built-in Theme of the same appearance while retaining the saved selection; resolving the conflict restores the intended Theme on the next launch.

**Startup fallback preserves the choice.** A missing or malformed selected Theme file produces a visible error and the default built-in Theme of the same appearance. The saved selection stays unchanged, including across autosave, so restoring or fixing the file restores the intended Theme on the next launch.

**Invalid appearance settings reject the document.** Unknown appearance keys and out-of-range widths reject the entire Theme document. The error identifies the offending setting and explains the valid key or range; values are neither silently ignored nor clamped. A failed web reimport preserves the previous valid document. Contrast warnings alone do not reject a document.

**Measure, do not assert an ordering.** `restyle-egui-console/02` set "values, not adjectives" for chrome and it holds. For glyphs it becomes a validator: an arbitrary Theme chooses its colours freely, so the floor is measured per Theme and reported, and the orderings the current tests pin are facts about Okabe–Ito rather than rules.

Validate the resolved Theme's text against actual composited backgrounds:
role backgrounds, selection, Cursor and Region fills, and panel/input backgrounds for
console text. Reports identify the role and painted state that fails. Transparent
fact channels expose underlying paint and are evaluated accordingly. Colour
distinguishability is a separate assessment, not a claim of this contrast report.

Okabe–Ito's Sequence colour is preserved as an explicit accepted contrast failure: `#0072B2` on `#000000`, approximately 4.05:1 against the unchanged 4.5:1 floor. The validator continues to report it. Tests over shipped Themes reject additional unrecorded failures rather than silently accepting them or lowering the floor.

**A decision that predates a branch is not a decision.** `feat/egui-theming` carries a complete hand-tuned light palette and a working per-theme mechanism, and neither was decided by an issue, recorded in `theme.md`, or reviewed against a capture. Neither is taken as decided. `04` treats the light palette as a starting proposal for the light built-in Theme.

## Scope

Delivery may be split across pull requests, but Theme switching is exposed only
when Source and chrome both follow the selected Theme. `06` may land as
foundation work; `03` connects chrome and prepares the pickers. `04` prepares the
existing light proposal as a complete Theme with contrast results and visual
captures, obtains user acceptance, then exposes the pickers and mode control. Every usable
intermediate state must remain coherent, including startup restoration and OS
appearance changes. Loading follows `03` so it cannot activate Source-only
theming.

In: the Theme format, the built-in Themes including a light one, loading custom Theme files authored outside Orcvs, the contrast validator, the dark/light/mode settings and their persistence across restart, and moving the Cursor Effect's motion into settings.

Out: an in-app Theme editor, colour pickers for authoring Themes, live authoring previews, and automatic creation from the current Theme. Making or changing a custom Theme is a file edit outside Orcvs.

Out: font choice, custom font files, font selectors and native/web font asset
loading, font sizes, spacing, corner radii, shadows and gradients. These are not
scheduled follow-ups or release requirements. The design
must avoid blocking their future implementation; it need not implement them in
advance.

Out: what Source Paint must distinguish, which is `syntax-highlighting`'s question — this effort decides where a role's colour comes from, never which role a Cell gets. Out too: image-comparison regression testing, which `console-testing` rules out and `restyle-egui-console/03` covers by review.
