# 07 — Resume: paint text that spells no Function as Ordinary, and close the 01–04 review

**What to build:** Text that spells no Function draws as Ordinary text in the base colour instead of the Diagnostic colour, the Ordinary default becomes the Cursor's default colour, and the gaps the review of `01`–`04` found are closed. Paused on 2026-09-17 so a full Function reference can be built first; the rule for such text was decided on 2026-09-19, below.

**Blocked by:** None — paused by choice, not by a ticket. Resume after the Function reference effort lands.

**Status:** needs-triage

## Where the work stands

Resolved and committed on branch `syntax-highlighting`, not pushed, no pull request: `01`–`06` and `08`–`10`. `11` is open (needs-triage). What remains here is the Ordinary default, the review follow-ups and the judgement calls below.

## Decided

- [x] Text that spells no Function paints in the Ordinary colour with the base font and no tint, not in the Diagnostic colour. See the decision below.
- [ ] The Ordinary default changes from `#FFFFFF` to `#EAEBE5`, the Cursor's default colour, as a fixed default (not linked to the live Cursor setting). Update the defaults table in `01`, the theme documentation, and the tests that pin it.

## Decided: the rule reads the claim, not a Leftover Char (2026-09-19)

The "Leftover Char" framing is dropped. The Language Map needs no new fact, because the Parser's claim already separates the two cases:

- An operand slot's Token is its parent signature's declared expectation. An operand claim with no Atom over written content is an Invalid Operand and draws Diagnostic.
- At every Expression start the Parser seeds `(Token::Function, None)` as the thing to try, not as anything declared. An operand slot only enters that branch when `is_function_next()` holds, so a Function claim with no Atom only arises at an Expression start. It means the Cells spell nothing: nothing was expected and nothing failed.

So there are two rules for a Function claim. A claim with an Atom paints Function on the Function tint. A claim with no Atom paints Ordinary with no tint, the same as an unclaimed Cell. `hi`, a written `07`, a lone `|`, the trailing `<` of `<<<` and the `v` of `value` all paint Ordinary. The half-typed spelling loses its warning, which is intended, since a spelling on its way to valid is not an error. No `lang`, `CONTEXT.md` or ADR change. This reverses `04`'s criterion for a written `07` and a lone `|`, and `04` records that. Inside a Reservation, `06`'s Output Portal paint still takes precedence.

Built in `console/src/style.rs::claim_paint` (the unbound `Token::Function` arm), pinned by `style::tests::an_unbound_function_entry_draws_ordinary_with_no_tint` and end to end from Source text by `paint::tests::text_that_spells_no_function_is_ordinary_while_an_invalid_operand_stays_diagnostic`.

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

**2026-09-18 — four follow-ups moved to `08`/`09`.** A design review found the console rebuilding the parser's per-slot entry from two scalar projections, `token_at` and `bound_at`. `08` puts the claim itself on the Render Frame. `09` replaces the overlapping Token matches with one decision function, which settles the Pending/Invalid conflation, the duplicate Token → colour lookup, the positional bools and the `Paint::derive` test path. Those four are struck above. The Leftover Char decision (since settled without a Leftover Char, 2026-09-19) and the other follow-ups stay here. ADR 0044 records the seam.
