# 07 — Resume: paint Leftover Chars as Ordinary, and close the 01–04 review

**What to build:** Plain text that no Expression can use draws as Ordinary text in the base colour instead of the Diagnostic colour, the Ordinary default becomes the Cursor's default colour, and the gaps the review of `01`–`04` found are closed. Paused on 2026-09-17 so a full Function reference can be built first; the reference is how the interactions below get explored before the open rule is decided.

**Blocked by:** None — paused by choice, not by a ticket. Resume after the Function reference effort lands.

**Status:** needs-triage

## Where the work stands

Resolved and committed on branch `syntax-highlighting`, not pushed, no pull request:

- `01` Source colours as Theme settings — 2b774de
- `02` Token-colour Fill tint — 80d907f
- `03` Pending Operand as its tint, no letter — a6f7459
- `04` Invalid Operand in the Diagnostic colour — c66da63

`05` (ready-for-human) and `06` (needs-triage) are untouched.

## Decided, not yet built

- [ ] Leftover Chars paint in the Ordinary colour with the base font and no tint, not in the Diagnostic colour.
- [ ] The Ordinary default changes from `#FFFFFF` to `#EAEBE5`, the Cursor's default colour, as a fixed default (not linked to the live Cursor setting). Update the defaults table in `01`, the theme documentation, and the tests that pin it.

## Open decision (blocks the Leftover Char build)

The Language Map has no Leftover Char today. The row walk starts a parse at every non-space Cell, so `hello`, a written `07`, and a lone `|` all record the same thing: one-Cell `Function` entries that bound no Atom. `Token::Char` is only a fallback for content the Language Map does not claim, which real Source never reaches, and no test produces it from Source text. So `02`'s Char exclusion and `03`'s Char reasoning cover a case that never happens, and the spec's Unclaimed → Leftover Char role has no record behind it.

Distinguishing a Leftover Char from an incomplete spelling needs a new fact recorded by the Language Map (a language change: `CONTEXT.md`, and an ADR if the vocabulary changes). Candidates:

- **First-character rule (recommended when paused):** a refused entry whose character can open a Function, Bang, or Comment spelling (`. ~ : & * ! ^ v < > |`) is an incomplete spelling and stays Diagnostic; any other refused entry is a Leftover Char. `07` and `hello` become Ordinary; `|`, `.`, and the trailing `<` of `<<<` stay Diagnostic. Oddity: the `v` in `value` stays Diagnostic because `v` opens `vv`.
- **Everything but `|`:** every refused entry except a lone `|` is a Leftover Char.

Either rule reverses `04`'s criterion that a written `07` draws in the Diagnostic colour; record that on `04` when decided. It also interacts with `05` (what a written Result parses as).

## Review follow-ups (from the two-axis review of 1c7aa98...c66da63)

Spec gaps:

- [ ] `04` paint tests start from Source text through the Render Frame to the glyph colour for `.+c40G`, `.+**01`, `.+||02`, a lone `|` (not `|a`), and `07`; `.+**01` asserts all four operand Cells; an invalid operand on the Cursor Cell is covered.
- [ ] `02` and `03` paint tests for Note, Atom, Sequence, 0%, and the Cursor Cell start from Source text where a Source can produce them, instead of hand-built Tokens.
- [ ] The "Comment is the dimmest glyph" rule is restated with named exceptions (Sequence, Diagnostic, Function, Bang are dimmer against `#000000`) rather than reduced to "dimmer than Ordinary", so Comment brightening past Number or Note fails a test.
- [ ] Stale doc references to nonexistent tests are fixed: the dimmest-glyph test cited in the contrast test's doc comment, and `a_nested_functions_own_cells_are_tinted_like_its_parents` cited beside the tint tests.

- [ ] `Paint::derive` still exists as a default-settings path, kept for `console/benches/paint.rs` (from `09`). Give the bench a way to build default colours, then remove or narrow it.
- [ ] `Paint::derive_with_colours` caches `slot_written` per claim in a `HashMap<*const Claim, bool>` inside the per-Cell loop (from `09`). Check the paint bench on the pull request; if it moved, answer "written" once per claim without hashing, e.g. on the Render Frame beside the claim.

Standards judgement calls (take or leave when resumed):

- ~~Pending and Invalid are conflated: `RenderCell::bound` is `Option<bool>` and answers `Some(false)` for an empty operand, hidden by `unwrap_or(true)` in the paint. Consider a small type. Note a half-typed operand (`.+0`) draws its `0` Diagnostic while being typed; the spec defines Pending as empty only, so that is currently correct but may read as noise.~~ Moved to 08, 09.
- ~~One Token → colour lookup on `SourcePaintSettings` replaces the duplicated glyph and tint matches.~~ Moved to 09.
- [ ] Bundle the Cursor effect sample, Cursor effect settings, and Source Paint settings threaded through `show_source`, removing its `too_many_arguments` allowance.
- ~~Drop `SourcePaintSettings::result` and its `dead_code` allowance until `06` paints it.~~ `06` now paints it (renamed to `output_portal`).
- [ ] Replace the ten repeated colour-picker blocks with one table.
- ~~The positional `bound, selected, cursor_visible` bools on the cell-visuals functions; tests pass `bound: true` for Cells that are unbound.~~ Moved to 09.
- ~~`Paint::derive` is `pub`, test-only, and paints default rather than live settings.~~ Moved to 09.
- [ ] egui's own background and error colours are seeded from static defaults while the Grid follows live settings; seed from the restored settings at startup.

## Comments

**2026-09-18 — four follow-ups moved to `08`/`09`.** A design review found the console rebuilding the parser's per-slot entry from two scalar projections, `token_at` and `bound_at`. `08` puts the claim itself on the Render Frame. `09` replaces the overlapping Token matches with one decision function, which settles the Pending/Invalid conflation, the duplicate Token → colour lookup, the positional bools and the `Paint::derive` test path. Those four are struck above. The Leftover Char decision and the other follow-ups stay here. ADR 0044 records the seam.
