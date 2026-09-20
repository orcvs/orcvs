# Theme the console

**Goal:** One answer to where every console colour comes from, so a theme can be switched, a default can be moved, and a viewer's preference can survive a restart without the three fighting each other.

## Why

`restyle-egui-console` decided the console's look and finished: `01` squared the Source Grid viewport and `02` pinned twenty-two palette tokens as the record `console/src/theme.md` keeps. What stayed open under that slug was not restyling. It was the mechanism — selecting a palette by theme, deciding a light one, keeping a restored preference, and seeding egui's own chrome. Those four moved here; `03` stayed behind because it is candidate visual verification that `v1-release/01` depends on, and it photographs the theme rather than deciding it.

The effort exists because colour authority is currently split and nothing records the split. `restyle-egui-console/02` names twenty-two tokens that a palette owns. `syntax-highlighting/01` shipped ten more — the Source background and every Token's glyph — as viewer settings that storage owns, under `Theme → Source colours`, and deferred the reconciliation to whoever came next. No ADR covers either claim. `01` settles it, and every other issue here inherits the answer.

## Rules

Authority is recorded before it is ported. `01` blocks nothing mechanically except `05`, but an issue that moves colour between mechanisms without the ADR is doing the reconciliation by accident. `03` in particular renames `PALETTE`, moves every line `restyle-egui-console/02` cites, and writes an ADR of its own; it should not also silently decide the question `01` asks.

A setting stores what the viewer chose, not what the theme resolved. This is the rule `01` exists to ratify and the one the current code breaks: `SourcePaintSettings` writes all ten resolved colours on every autosave, so storage holds a snapshot of the theme and no later default or palette can reach that install. Whatever `01` decides, a settings layer that cannot represent "unset" cannot coexist with theme switching.

Values, not adjectives, and a measured ratio rather than an eye. `restyle-egui-console/02` set this for dark and `04` inherits it for light: every token is named at an exact value in `console/src/theme.md`, and every glyph-on-background contrast is stated as a number. `console-testing/03` pins the values so the record and the code cannot drift.

A decision that predates a branch is not a decision. `feat/egui-theming` carries a complete hand-tuned light palette and a working per-theme mechanism, and neither was ever decided by an issue, recorded in `theme.md`, or reviewed against a capture. `03` takes the mechanism as a port rather than a merge, and `04` treats the palette as a starting proposal.

## Scope

In: selecting a palette by resolved theme, the light palette's values, the theme preference across restart, egui chrome that agrees with the Grid on first paint, and where Source Paint colour authority lives.

Out: what Source Paint must distinguish, which is `syntax-highlighting`'s question — this effort decides where a role's colour comes from, never which role a Cell gets. Out too: image-comparison regression testing, which `console-testing` rules out and `restyle-egui-console/03` covers by review.
