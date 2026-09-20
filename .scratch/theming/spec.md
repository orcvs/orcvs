# Theme the console

**Goal:** Syntax highlighting that is a themeable option, in a format the rest of the world already publishes themes for, with console chrome themed separately by egui's own model.

## Why

`restyle-egui-console` decided the console's look and finished: `01` squared the Source Grid viewport and `02` pinned the palette tokens `console/src/theme.md` records. What stayed open under that slug was not restyling but theming — selecting a chrome palette by theme, deciding a light one, keeping a restored preference, and the Source colours themselves. Those moved here; `restyle-egui-console/03` stayed behind because it is candidate visual verification `v1-release/01` depends on, and it photographs the theme rather than deciding it.

`syntax-highlighting/01` shipped `Theme → Source colours` — ten resolved colours in a settings value, persisted in full on every autosave — and deferred reconciling it with the palette. ADR 0051 is that reconciliation.

## Rules

**A Cell carries facts. The console draws them through channels. A theme is the mapping.** Facts are the parser's claim, whether a slot is written, whether a Cell is in an Output Portal Reservation, whether it is selected. Channels are glyph colour, fill, border, overlay. Which channel shows which fact is a theme decision, and `style.rs` stops making it in control flow.

**Syntax and affordance are different namespaces.** A Token is a spelling the Language Map answers and takes a base16 slot. Diagnostic, Output Portal, Fill tint, Region, selection and the Cursor are console affordances and take named keys. This is VS Code's split between `tokenColors` and the `editorError.*` workbench keys, and it is why a Diagnostic never competes with a Token for a slot.

**Base16, because the schemes already exist.** The value of a theme format is the several hundred published schemes that target it. A shape of our own would be more precise about Orcvs's seven Tokens and would accept no scheme anyone has written. Precision that nobody else can use is not precision worth having.

**A theme stores what was chosen, never what was resolved.** A scheme name or its sixteen values, plus overrides keyed by scheme. Nothing stores a resolved colour, so a template change, an improved scheme, or a moved default reaches every install. This is the property `SourcePaintSettings` lacks and the reason it is replaced rather than migrated.

**Glyph colour is a language concept; chrome is the toolkit's.** `03` carried this claim before ADR 0051 recorded it. The console's presentation tokens are conceded to `Visuals` and resolve by `ctx.theme()`. The Source scheme does not flip because the menu bar did.

**Measure, do not assert an ordering.** `restyle-egui-console/02` set "values, not adjectives" for chrome and it holds. For glyphs it becomes a validator: an arbitrary scheme orders its accents however it likes, so the floor is measured per scheme and reported, and the orderings the current tests pin are facts about Okabe–Ito rather than rules.

**A decision that predates a branch is not a decision.** `feat/egui-theming` carries a complete hand-tuned light palette and a working per-theme mechanism, and neither was decided by an issue, recorded in `theme.md`, or reviewed against a capture. `03` takes the mechanism as a port rather than a merge; `04` treats the palette as a starting proposal.

## Scope

In: the theme format and its two namespaces, the Source scheme and its loader, the contrast validator, chrome palette selection by theme, the light chrome values, and the theme preference across restart.

Out: what Source Paint must distinguish, which is `syntax-highlighting`'s question — this effort decides where a role's colour comes from, never which role a Cell gets. Out too: image-comparison regression testing, which `console-testing` rules out and `restyle-egui-console/03` covers by review.
