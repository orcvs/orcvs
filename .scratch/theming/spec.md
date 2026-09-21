# Theme the console

**Goal:** Every part of the console's presentation is themeable, and one Theme styles all of it: the Source Grid and the console around it, from a format the rest of the world already publishes themes for.

## Why

`restyle-egui-console` decided the console's look and finished: `01` squared the Source Grid viewport and `02` pinned the palette tokens `console/src/theme.md` records. What stayed open under that slug was not restyling but theming — selecting a chrome palette by theme, deciding a light one, keeping a restored preference, and the Source colours themselves. Those moved here; `restyle-egui-console/03` stayed behind because it is candidate visual verification `v1-release/01` depends on, and it photographs the theme rather than deciding it.

`syntax-highlighting/01` shipped `Theme → Source colours` — ten resolved colours in a settings value, persisted in full on every autosave — and deferred reconciling it with the palette. ADR 0051 was that reconciliation, and ADR 0053 corrects it: ADR 0051 conceded chrome to egui's dark/light `Visuals`, which left most of the console outside any theme. The requirement was always that all presentation is themeable.

## Rules

**A Cell carries facts. The console draws them through channels. A theme is the mapping.** Facts are the parser's claim, whether a slot is written, whether a Cell is in an Output Portal Reservation, whether it is selected. Channels are glyph colour, fill, border, overlay. Which channel shows which fact is a theme decision, and `style.rs` stops making it in control flow.

**One Theme, sixteen slots plus named keys.** A Token takes a base16 slot. Everything else the console draws — Diagnostic, Output Portal, Fill tint, Cursor, Region, grid lines, Sector Seams, panels, text, selection, error and warning — takes a named key whose default comes from a slot, so a bare published scheme styles the whole console. ADR 0053 lists the keys. Diagnostic and Output Portal take a key per channel, as VS Code's `editorError.*` does, so a Theme picks a fact's channel through colour values rather than a mapping.

**Base16, because the schemes already exist.** The value of a theme format is the several hundred published schemes that target it. A shape of our own would be more precise about Orcvs's seven Tokens and would accept no scheme anyone has written. Precision that nobody else can use is not precision worth having.

**Settings name Themes; Themes are documents; there are no overrides.** Settings hold a dark Theme, a light Theme, and a mode (follow the OS, or hold one). A different look is a custom Theme that inherits from one built-in Theme and lists what it changes. Nothing stores a resolved colour the viewer did not choose, so an improved built-in Theme reaches every install. This is the property `SourcePaintSettings` lacks and the reason it is replaced rather than migrated.

**A Theme decides how things look, never how much they move.** Glitch amount and frequency are settings. egui's `Visuals` are built from the resolved Theme; they are an output of it, never a second authority.

**Measure, do not assert an ordering.** `restyle-egui-console/02` set "values, not adjectives" for chrome and it holds. For glyphs it becomes a validator: an arbitrary scheme orders its accents however it likes, so the floor is measured per scheme and reported, and the orderings the current tests pin are facts about Okabe–Ito rather than rules.

**A decision that predates a branch is not a decision.** `feat/egui-theming` carries a complete hand-tuned light palette and a working per-theme mechanism, and neither was decided by an issue, recorded in `theme.md`, or reviewed against a capture. Neither is taken as decided. `04` treats the light palette as a starting proposal for the light built-in Theme.

## Scope

In: the Theme format, the built-in Themes including a light one, custom Themes and their editor, loading a base16 scheme, the contrast validator, the dark/light/mode settings and their persistence across restart, and moving the Cursor Effect's motion into settings.

Out: what Source Paint must distinguish, which is `syntax-highlighting`'s question — this effort decides where a role's colour comes from, never which role a Cell gets. Out too: image-comparison regression testing, which `console-testing` rules out and `restyle-egui-console/03` covers by review.
