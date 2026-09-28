# 17 — Correct console comments the source-audit changes leave behind

**What to build:** After source-comments/04 holds `console` to `docs/agents/comments.md` (lineage, provenance citations — including the 139 `.scratch/` citations — and the gate carve-outs), what remains is the work that rule does not cover and that depends on the code the console source-audit tickets change: upstream citations that name a file line rather than the behaviour relied on (29 `file.rs:NN` citations in `.rs` files, 26 of them in `console.rs` and its `console/` submodules; the 8 in `console/Cargo.toml` are source-comments/05's), and explanations repeated within a single function. About 24% of console `.rs` lines are comments.

**Blocked by:** source-comments/04; 11 — Decide the fate of the colour-blindness simulation; 13 — Split Console::ui into panel modules; 14 — Drive the panel layout test through Console::ui; 18 — Make TOML the only Theme representation; 22 — Keep an earlier refused Source when a later start also refuses (wontfix, #170).

**Status:** resolved

- [x] Upstream citations name the behaviour relied on, not a file line.
- [x] Each argument appears once, at the item that enforces it.
- [x] Rendering-budget and invariant explanations are kept.
- [x] Comments that contradict the code after the tickets above land are corrected or removed.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** The lineage and citation criteria duplicated source-comments/04, and the manifest criterion duplicated source-comments/05 (both filed by 0ce8d2c4). This ticket keeps what those do not cover. The earlier "about 30%" did not reproduce; about 24% then and now.

**2026-09-27 — resolved in [#175](https://github.com/orcvs/orcvs/pull/175)** (commit "State each console argument once and correct comments the code contradicts"). All 29 upstream `file.rs:NN` citations in `.rs` files now name the item relied on, checked against the vendored 0.36.2 sources; `glyphs.rs` had named the wrong function (`FontFace::styled_metrics` does the scaling). File-only citations with no line number are kept. Repeated arguments are kept at the enforcing item and cut to a pointer elsewhere, in `style.rs`, `theme.rs`, `shapes.rs`, `cursor_effects.rs`, `input.rs`, `glyphs.rs`, `function_reference.rs`, `paint.rs`, `contrast.rs`, `colour_vision.rs`, `kittest_tests.rs` and `benches/paint.rs`. Rendering-budget and invariant explanations are kept. Contradictions corrected: `native_midi::AVAILABLE` called "a runtime answer" (it is a const); "Refresh" for the menu item labelled "Scan" (`console.rs`, `panel.rs`, `midi.rs`); stroke widths said to scale with zoom (`grid_viewport.rs`, `shapes.rs`); egui-template font comments that called a first-inserted font a fallback; `is_presentable`'s inverse attributed to a Scene; lang wording `contrast.rs` quoted that lang no longer has; "six" function-reference headers (seven) and a dynamic-Function list missing Random; a chrome width test said to retune five keys (four); `paint.rs` test docs describing a background mix `style` does not perform; a Cursor wake given as 45–190 ms (it is sub-second). Only comment lines changed.
