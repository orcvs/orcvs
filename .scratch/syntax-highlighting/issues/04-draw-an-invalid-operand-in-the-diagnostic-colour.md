# 04 — Draw an Invalid Operand in the diagnostic colour

**What to build:** When an operand's Cells fail to bind as its declared Token, those Cells keep their Token tint and draw their glyphs in the Diagnostic colour. A lone `|` draws in the Diagnostic colour too.

**Blocked by:** 02 — Tint Functions and Operands by their Token colour.

**Status:** resolved

**Tags:** release/v1

- [x] `.+c40G`: both operands keep the Number tint; `c4` and `0G` draw in the Diagnostic colour.
- [x] `.+**01` and `.+||02`: the rejected `**` and `||` keep the Number tint and draw in the Diagnostic colour; `01` and `02` are unchanged.
- [x] A Function entry that bound no Atom — a refused Function spelling — draws in the Diagnostic colour with no tint. This covers a lone `|`, both Cells of a written `07`, and the trailing `<` of `<<<`, with no spelling-specific case for `|`. **Reversed 2026-09-19 (`07`):** an unbound Function claim now paints Ordinary with no tint; Diagnostic is kept for operand slots only.
- [x] The fact comes from the Parser's own record that the operand bound no Atom, exposed through the Language Map to the Render Frame. The Expression-wide Diagnostic Span and the generic one-Cell lexical diagnostic are not used to infer it, and no new per-Cell classification vocabulary is minted.
- [x] A valid Operand beside an invalid one in the same Expression is unaffected.
- [x] Evaluation-time operand diagnostics (e.g. a zero divisor) are out of scope: they are Tick outcomes, not Source facts.
- [x] Focused tests cover each case above at the Render Frame and at the paint.

## Comments

The Token stays the declared one: `c4` in a Number slot is still Number, which is why the tint stays.

**Refused Functions (decided 2026-09-17).** The Parser records a lone `|` as `(Token::Function, atom: None)`: every unit starts as a Function slot, `|x` fails `Function::try_from`, and the refusal advances one character (ADR 0018). Nothing distinguishes it from any other refused spelling, so the rule is general: an unbound Function entry is Diagnostic with no tint, because its `Function` label records what the slot expected, not what was found. Unbound operand entries differ — they keep their declared Token tint. `name_units`' per-character "invalid Language Unit character" lexical diagnostic is not the source of this fact.

**Verified every parse before writing a test against it.** A probe against `LanguageMap::build` (written, run, then discarded — never shipped) confirmed each case's positioned entries before its test was written: `.+c40G` records `(Function, bound, 0..2)`, `(Number, unbound, 2..4)`, `(Number, unbound, 4..6)`; `.+**01` and `.+||02` both record their first Number operand unbound and their second bound (neither `**` nor `||` is recognized as "next is a Function" at an operand position, so both are read and refused as Number literals, not treated as a nested Bang or Comment); `|a` records two one-Cell unbound Function entries (`|` then `a`); `07` records two one-Cell unbound Function entries the same shape as `|`; and `<<<` records one bound two-Cell Function entry (`<<`, Self-Banging West) followed by one one-Cell unbound Function entry (the trailing `<`) — confirming the ticket's `07` and `<<<` claims exactly. `<<<` is not one of the shipped test cases: it is structurally `07` with its first entry bound instead of unbound, so the probe was enough to confirm the general rule needs no spelling-specific case, without shipping a redundant test.

**Where the bound/unbound fact is exposed.** `LanguageMap` gains a sibling query, `bound_at`, beside `token_at`; both now read one shared `entry_at` lookup instead of walking the rows twice. `bound_at` answers `entry.token == Token::Comment || entry.atom.is_some()` — the Comment exception lives here, in the Language Map, matching `name_units`'s own established precedent of checking a Comment's Token before its Atom, rather than being left for the console to rediscover from Token alone. `RenderCell` carries the answer as `bound: Option<bool>` beside `token()`, populated straight from `source.language_map().bound_at(position)` in `RenderFrame::derive`, the same way `expressions()` and `lexical_diagnostics()` already reach through `language_map()` directly rather than through a `SourceRevision` wrapper — a wrapper was not needed here because, unlike `token_at`, `bound_at` has no leftover-Char fallback to apply.

**Where the fact is spent.** `style::cell_visuals_with_cursor_colour` and `style::fill_tint_colour` both gain a `bound: bool` parameter (the caller folds `RenderCell::bound()`'s `None` — a Cell no entry claims — into `true`, since nothing there can be marked invalid). The foreground match reads `bound` only for `Function`, `Number`, `Note`, `Atom`, and `Sequence` — the Tokens a refusal can reach — and answers `source_paint.diagnostic()` for an unbound one instead of its Token colour; every other Token and an unclaimed Cell ignore `bound` outright, pinned by `style::tests::bound_is_read_only_for_the_tokens_a_refusal_can_reach`. `fill_tint_colour` reads `bound` only in its `Token::Function` arm — an unbound Function paints no tint — leaving every Operand Token's tint unconditional, exactly as `syntax-highlighting/02` left it. A Pending (still-empty) operand Cell answers `bound: Some(false)` by the same Parser fact as a written Invalid one (`take_token` on blank Cells fails to decode, same as on wrong content), but draws no visible change: `paint.rs`'s `character: cell.content().unwrap_or(' ')` already leaves it blank regardless of foreground colour, so only its tint shows, unchanged from `syntax-highlighting/03`.

`console/src/source_paint.rs`'s `#[allow(dead_code)]` on `SourcePaintSettings::diagnostic` is removed now that `style.rs` reads it; the one on `result` stays, reserved for `syntax-highlighting/06`. `console/src/theme.md` documents the Diagnostic role's painting and the Fill tint exception for a refused Function.
