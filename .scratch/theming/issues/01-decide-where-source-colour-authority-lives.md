# 01 — Decide where Source colour authority lives

**What to decide:** Whether the ten Source Paint roles are viewer settings that storage owns, or theme tokens that the palette owns, and record the answer as an ADR. Every other issue in this effort inherits the answer.

**Blocked by:** None — can start immediately.

**Status:** ready-for-human

## Why this is open

`syntax-highlighting/01` shipped `Theme → Source colours`: one opaque colour picker per Source Paint role, a reset, and persistence under its own key. It resolved on 2026-09-17. Its own comments deferred the conflict rather than settling it — "This supersedes the palette-selection part of `restyle-egui-console/04` for Source glyph colours... The light palette in `restyle-egui-console/05` is out of scope. Those tickets are not edited here; reconcile them when this lands." It landed. The reconciliation is this issue.

Two mechanisms now claim the same colours. `restyle-egui-console/02` decides twenty-two palette tokens as the console's recorded record, and `03` of this effort ports the per-theme mechanism so `Visuals` derives "from a palette selected by theme rather than from one hardcoded const". Meanwhile `SourcePaintSettings` (`console/src/source_paint.rs:74`) holds ten resolved `Color32` fields that persistence writes out in full on every autosave (`console/src/persistence.rs:126`) and `starting_source` restores verbatim (`:175-179`).

## What the current shape costs

`SourcePaintSettings` stores resolved colours, not overrides. There is no way to represent "this role is whatever the theme says", so every autosave writes all ten whether or not the viewer touched any. Storage ends up holding a snapshot of the theme rather than a deviation from it, and the constants in `source_paint.rs` are dead to that install from its first save onward.

Three consequences, in increasing order of what they cost:

- A default change never reaches an existing install. `syntax-highlighting/07` moved Ordinary from `#FFFFFF` to `#EAEBE5` so plain Source text and the Cursor frame read as one white. Any install that has run once already holds `255,255,255` explicitly, with no migration and no unmodified marker, so the new default and its 17.51:1 figure apply only to a fresh install or cleared storage.
- "Reset to theme defaults" (`console/src/console.rs:2004`) does not restore theme authority. It copies the current defaults into the struct, which the next autosave writes back as literals — so a viewer who explicitly asked to return to theme defaults is pinned to the values that happened to be current when they clicked.
- `04` cannot reach those roles at all. When the light palette lands, every existing install holds ten dark values in storage, and no light token can apply to any of the ten. A settings layer that stores resolved values is incompatible with theme switching, which is the whole point of `03` and `04`.

The same flaw sits in `CursorEffectSettings` (`console/src/cursor_effects.rs:14`) for its five non-optional fields. That struct already uses `Option<Color32>` for `cell_colour` and `region_cursor_colour`, so the shape this issue may want is half-built next door.

## The candidate answers

**Overrides over a theme-selected palette.** Each role becomes `Option<Color32>`; accessors resolve `None` against the palette for the resolved theme; persistence writes only the `Some` values; reset writes `None` rather than the defaults. The theme stays authoritative for anything untouched, and a default or palette change lands on everyone who did not override it. `05` survives with its acceptance lines reworded to name the resolved palette rather than the restored settings.

**Theme tokens, no editor.** The ten roles join the other twenty-two as palette tokens, the `Theme → Source colours` section and the `source_paint` storage key are deleted, and `style()` reads them from the theme palette directly. `05` becomes moot and is absorbed by `03`.

Either answer needs a decision about existing stored values. The current encoding is ten `r,g,b` groups joined by `;` with no room for "unset", so an existing string decodes under any new format as ten deliberate overrides — preserving exactly the behaviour being removed. Changing the storage key and letting the old one go unread is the cheap migration: `decode` already refuses whole and falls back to defaults, and the repo has no public compatibility contract and no external installs.

## Acceptance

- [ ] An ADR records which mechanism owns Source Paint colour, and why. Note that `restyle-egui-console/04` — now `03` of this effort — carries an acceptance line reading "ADR 0030 is written", which is stale: 0030 is taken (`docs/adr/0030-terminal-output-functions-extend-over-sequences.md`) and the next free number is higher. Settle the number against `.scratch/adr-numbering` when writing.
- [ ] The ADR states what happens to values already in storage, and whether the `source_paint` key is migrated, re-keyed or dropped.
- [ ] The ADR says whether the same answer binds `CursorEffectSettings`, or whether Cursor effects are deliberately left as resolved values.
- [ ] `03`, `04` and `05` of this effort are edited to match the decision, or explicitly confirmed as unaffected.
- [ ] `console/src/theme.md`'s "Source colours" section is corrected to describe whatever the decision makes true.
- [ ] `syntax-highlighting/01` gains a comment pointing at the ADR, so the deferred reconciliation is visibly closed rather than silently abandoned.

## Comments

**2026-09-20 — opened from triage of `05`.**

> *This was generated by AI during triage.*

`05` could not be triaged on its own terms: it presupposes that persisted `SourcePaintSettings` is the right source of truth for chrome, which is the question here. `05` is recorded as blocked by this issue.

`ready-for-human` because it is a design decision about where authority lives, and because the second candidate answer deletes a shipped UI surface — neither is a call to delegate.

No ADR covers this today. There are 49 ADRs and none mentions Theme settings, Source colour configuration, or a persisted preferences surface. Two supporting observations about how that happened, offered as evidence rather than as blame: `syntax-highlighting`'s `spec.md` opens "What Source Paint must distinguish. Not colours, not Cursor, not Sector Seams", and the commit that created that spec (`1baa354`) created the colour-editor ticket alongside it the same day the implementation landed (`910f439`); and the precedent that ticket cites — `Theme → Cursor effects`, from `.scratch/cursor-effects/issues/03-tune-cursor-effects-through-theme-settings.md` — comes from an effort with no `spec.md` at all.
