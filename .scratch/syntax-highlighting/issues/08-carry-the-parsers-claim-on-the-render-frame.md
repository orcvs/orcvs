# 08 — Carry the parser's claim on the Render Frame

**What to build:** A Render Frame Cell answers the parser's claim on it: the claimed Cell range, the declared Token, and the Atom the Cells bound, or none. Today the frame gives out two scalar projections of that entry, `token` and `bound`, and the entry is rebuilt from them at six places: three in `orcvs` and three in `console`. This issue carries the claim and removes the `bound` projection. `09` changes the paint decision. Built on ADR 0044.

**Blocked by:** None — can start immediately.

**Status:** resolved

**Tags:** release/v1

- [x] `orcvs` gains a `Claim` carrying `cells`, `token` and `atom` exactly as `lang::PositionedEntry` records them. It does not carry `parent`: that index counts entries within one Expression (`lang/src/expression.rs:16-18`), so it means nothing outside it.
- [x] `RenderCell::claim() -> Option<&Claim>` answers the claim on that Cell; `None` means no entry claims it. Each claim is stored once and shared by every Cell it claims, not cloned per Cell. `RenderFrame` already stores a flat `Vec<RenderCell>` (`render_frame.rs:85`).
- [x] `RenderCell::bound()` and `LanguageMap::bound_at` are removed, and their callers move to the claim in this issue. The production callers are `RenderFrame::derive` and `paint.rs:214`; `paint.rs:486` is a test. Moving `paint.rs` is mechanical, with no behaviour change: the bound value `bound().unwrap_or(true)` supplied becomes `cell.claim().is_none_or(|claim| claim.token == Token::Comment || claim.atom.is_some())`, passed to the existing `bound` parameter. The Comment clause is required: a Comment records no Atom, and `bound_at` answered `true` for it. It is temporary, and goes when `09` decides by Token. `09` then restructures the paint decision itself.
- [x] `RenderCell::token()` stays in this issue. `paint.rs:213` still reads it, and `09` removes it. `SourceRevision::token_at` and `LanguageMap::token_at` are unaffected.
- [x] The derivation lives inside `orcvs::source`, the one module that can reach `ExpressionEntry::positioned()` (`pub(super)`, `language_map.rs:158`). No visibility is widened for it.
- [x] Tests drive Source text through a `SourceCommander` to a Render Frame and assert the claim for: `.+c40G` (a valid Function claim, two unbound Number claims), `.+01  ` (a bound Number and an unbound claim over two blank Cells), a row-truncated operand, a lone `|`, `07` (two one-Cell unbound Function claims), a Comment, a standalone `**`, and an unclaimed Cell (`None`). They replace the `bound()` assertions in `render_frame.rs` (`:424-509`).
- [x] ADR 0040's claim that `RenderFrame` stores `Vec<Vec<RenderCell>>` and "is not flattened to match" (`docs/adr/0040-the-console-paints-from-a-value.md:37`) is corrected to the flat storage.

## Answer

A Render Frame Cell now answers `claim() -> Option<&Claim>`. `Claim` is `cells`, `token`, and `atom` — `lang::PositionedEntry` without `parent`. `LanguageMap::claims_by_cell` derives it inside `orcvs::source` from `ExpressionEntry::positioned()`, still `pub(super)`. Each claim is one `Arc`, shared by every Cell it covers. `bound()` and `bound_at` are gone. Paint folds the claim with the Comment clause the ticket named; that fold is temporary until `09`.

## Comments

Pending and Invalid are not told apart here. `atom: None` covers both of them. The claim carries its Cell range, so `09` can read the content of the whole slot. A two-Cell claim holding one blank Cell and one written Cell is an invalid slot, so checking only the blank Cell gives the wrong answer.

`LanguageUnitKind`, `LanguageMap::units()` and the Leftover Char fallback in `SourceRevision::token_at` are left alone. A pending slot forms no Language Unit (`language_map.rs:558`, `:574-583`), so units cannot be where the claim comes from.

Gates: `cargo fmt --all -- --check`; clippy and nextest scoped to `orcvs`, then to `console`.
