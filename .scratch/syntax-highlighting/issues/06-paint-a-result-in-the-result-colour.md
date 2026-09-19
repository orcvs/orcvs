# 06 — Paint an Output Portal in the Output Portal colour

**What to build:** Cells the decision in `05` names as an Output Portal draw their glyph in the Output Portal colour on an Output Portal tint, so a written value stands apart from the Token-tinted Expression that produced it.

**Blocked by:** 01 — Expose Source colours as Theme settings; 05 — Decide how a Result is known before a Tick; 09 — Paint from the claim; 10 — Derive Output Portal destinations and prove agreement with the scheduler. (Uses the Fill tint from 02.)

**Status:** resolved

- [x] A scalar answer (`07` south of `.+0304`, `C4` south of `.^3C`) draws in the Output Portal colour on an Output Portal tint at the Fill tint strength.
- [x] A Sequence answer (`04030201` south of `:<:-0104`) is painted the same way across all its Cells, and the Reservation's remaining Cells to the end of the row are painted as empty Output Portal Cells.
- [x] A Bang answer (`**` south of `~*0401`) keeps the Bang glyph colour on the Output Portal tint.
- [x] A Terminal Output Function, Halt, and a Source-writing Function paint nothing at their Output Portal.
- [x] An empty Output Portal Cell shows the Output Portal tint and no glyph, including before the first Tick.
- [x] The paint precedence where an Output Portal covers another Expression's claimed Cells (a consumer's operand, another root) is decided and pinned by a test.
- [x] The paint role, Theme colour label, `SourcePaintSettings::result` and its accessors, `DEFAULT_RESULT`, and `console/src/theme.md` say Output Portal. The stored format is positional and does not change.
- [x] Focused paint tests cover each case.

## Answer

`console/src/style.rs::claim_paint` reads `RenderCell::output_portal()` as a third, explicit input alongside the claim and the slot's written fact — the same one decision `09` introduced, not a second decision path. `output_portal` is read first: when it is `true`, `claim_paint` returns the Output Portal colour on the Output Portal's own Fill tint (`fill_tint_colour(source_paint.output_portal(), source_paint)`) for every claim shape — an unclaimed empty Cell, an unbound refused Function claim (what a written scalar or Sequence answer re-parses as, `05`'s Answer), and a bound Operand claim alike — with two named exceptions:

- **A bound Function claim** (`Token::Function`, `atom: Some(_)`) keeps its ordinary Function paint outright, whatever `output_portal` says. This is the precedence decision the paint precedence checkbox asks for. It covers both named overlaps from `05`'s Overlap rule in one rule: a Cell that is another Expression's own two-Cell Function spelling ("another root") is never repainted, because every Function's own spelling carries `Token::Function` regardless of nesting — the rule needs no Expression lookup to tell a root's spelling from a nested one, which is why it is the smallest coherent choice. Every other overlapping Cell, including another Expression's own Number, Note, Atom or Sequence operand ("a consumer's operand"), takes the Output Portal colour and tint instead of its declared role, because the Reservation's answer is what a viewer reads there.
- **A bound Bang claim** keeps its own Bang glyph colour (`source_paint.bang()`) rather than the Output Portal colour, because a Bang is what a Producer emits rather than a value it writes — but it still takes the Output Portal's own Fill tint in place of Bang's usual bare `None`, so a Bang answer still reads as an Output Portal Cell.

The Cursor's own fill is unaffected: `cell_visuals_with_cursor_colour` still applies it after `claim_paint`, so it wins outright over Output Portal exactly as it already won over the Fill tint.

`console/src/paint.rs::Paint::derive_with_colours` passes `cell.output_portal()` through to `cell_visuals_with_cursor_colour` alongside the claim and `slot_written`'s answer — one more fact read once per Cell, at no extra derivation cost, since `RenderCell::output_portal()` (`10`) is already a plain per-Cell `bool`.

**Renaming.** `SourcePaintSettings::result`/`result_mut`/`DEFAULT_RESULT` are `output_portal`/`output_portal_mut`/`DEFAULT_OUTPUT_PORTAL`; the `#[allow(dead_code)]` on the accessor is gone now that `claim_paint` calls it. The Theme colour picker label is "Output Portal". The stored format is unchanged: `encode`/`decode` are positional (`syntax-highlighting/05`'s Answer), so the tenth `r,g,b` group means the same thing under its new Rust-side name. `source_paint::tests::a_previously_stored_settings_string_still_decodes_to_the_same_colours` decodes a literal captured from the pre-rename `encode()` output and checks it still equals `SourcePaintSettings::default()`.

**Tests.** `console/src/paint.rs`'s `output_portal_paint` module builds every acceptance case from Source text through the Render Frame, writing the answer directly into Source with no Tick (the highlight comes from the current revision alone, `05`'s Answer): a scalar answer (`07` south of `.+0304`, and `C4` south of `.^3C`), a Sequence answer with its unfilled remainder (`04030201` south of `:<:-0104` on a wider row), a Bang answer contrasted against an ordinary untinted Bang elsewhere (`**` south of `~*0401`), an empty Output Portal before any Tick, a Terminal Output Function (`!>`), Halt (`*!`), a Self-Banging Function's Advance path (`>>`), and the overlap precedence (`:-0102` sequence-capable over `.+0304`, whose own Function spelling wins while its Number operands take the Output Portal). `console/src/style.rs`'s tests pin the same precedence directly against `claim_paint`/`cell_visuals_with_cursor_colour` with hand-built claims, including that the Cursor still wins over Output Portal.

**Visible output beyond Output Portal Cells.** None. No existing acceptance case from `01`–`04`/`09` changes: `output_portal` is `false` everywhere a Tick has not run one of these new fixtures, and the rename touches only Rust identifiers, a UI label, and documentation.

## Comments

Status stays `needs-triage` until `05` is decided, since the acceptance above assumes position-based Result Cells and `05` may change that.

**2026-09-18.** This issue adds the destination input to `09`'s paint decision. `09` leaves it out deliberately, because nothing could supply it before `10`.

**2026-09-19.** `05` resolved: the role is the Output Portal, highlighted from the current revision over the Reservation, parse unchanged. Criteria updated to match, and status moves to ready-for-agent.
