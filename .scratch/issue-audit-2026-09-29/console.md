# Console issue implementation audit — 2026-09-29

Audited working tree at `c34bccb6`; read implementation and assertion bodies, not just
prior issue comments. Skills: rust-review and egui. This is an issue audit, with no
Rust edits or claim of live visual verification. Line numbers name the audited tree.

## Open issues

| Issue | Verdict and actual evidence | Tests found (execution recorded below) |
|---|---|---|
| console-testing/02 | Keep open. `CONTEXT.md:283-329` defines Cursor through Panel, uses console repeatedly, and has no Console entry. | Documentation gap; no executable test needed. |
| theming/10 | Partial implementation; keep open behind `17`. Corrected one false unchecked criterion: `theme.rs:1150-1155` clones the parent and mutates supplied properties only; `:1590-1605` changes GridBackground and asserts inherited panel_background unchanged. Native file discovery is `theme_registry.rs:120`; built-in-only parent resolution `theme.rs:1129`; config selection is `config.rs:158`. Web import remains explicitly deferred. | `explicit_color_overrides_the_parent`, `explicit_transparent_colour_is_a_supplied_value_not_a_clear` (`theme.rs:1927`); appearance mismatch/match (`:1712/:1731`), omitted properties (`:1574`), duplicate stems (`theme_registry.rs:850`), missing/repaired selections (`:1181/:1209`), contrast notice (`:580`). |
| theming/12 | Keep ready-for-human. `git branch --list feat/egui-theming` finds the branch; `git worktree list` finds no worktree for it. No archive/delete decision inferred. | Git inspection only; no deletion performed. |
| theming/13 | Keep open. Seam colour still unpremultiplies per call (`style.rs:552-554`, called in `paint.rs:303/309`); optional Region Cursor fallback duplicated (`paint.rs:242`, `contrast.rs:344`). Viewport math contains no Theme border input, but no dedicated cross-border-width hit-test at multiple zooms exists. | Border painting tests exist (`console/tests.rs:3695`, widths independently applied); these do not assert hit testing across widths. |
| theming/14 | Keep open. ADR 0053 still promises pickers (`docs/adr/0053-one-theme-styles-the-whole-console.md:106-118`) and lacks its requested colour-vision decision. ADR 0044 still opens “superseded by ADR 0050” (`docs/adr/0044-paint-reads-the-parsers-claim.md:3`) despite later rejection. | Documentation findings. |
| theming/15 | Keep open for remaining stale benchmark names: `benches/floors.toml:63`, `.scratch/paint-cell-cost/spec.md:3,17` still name `derive_with_colours`; implementation is `Paint::derive_with_theme` (`paint.rs:191`). Already completed menu-test criterion is supported by `kittest_tests.rs:275`, which enumerates actual menus. | `no_menu_offers_a_setting` found; not independently executed here. |
| theming/16 | Keep open. Storage helper exists (`persistence.rs:143-224`), but tests persist Source rather than egui ThemePreference. `console/tests.rs:1081-1109` and `kittest_tests.rs:570` seed preference directly into a context, bypassing codec. | Existing construction/preference tests do not meet save/read/fresh-context/Console::new sequence. |
| theming/17 | Keep open. Paint Region cursor test (`paint.rs:715-765`) covers None and nontransparent supplied fill, not `Some(TRANSPARENT)`. Grid test (`console/tests.rs:3232-3255`) only asserts `Frame.fill` equality, despite its compositing name; no composited pixel assertion. | Resolver transparent-value test (`theme.rs:1927`) is insufficient for paint integration. |
| theming/18 | Keep open. `theme.md:7` assigns every value to dark theme although light is documented; `:300/:362` say five hues; `:731-733` claims Cursor/Region fills unset. ADR 0053 `:67-85` still records accepted Sequence/awaiting dark failures. | Built-in Theme and contrast tests establish shipped values; documentation still disagrees. Human amendment review remains requested by issue. |
| render-frame-responsibility/09 | Keep needs-triage. `Paint::background_runs` allocates runs and walks derived cells on demand (`paint.rs:475-514`); shape consumer still subtracts one from half-open end (`console/shapes.rs:168-181`). No evidence decision was made to move fold into derive. | Existing background-run tests (`paint.rs:2458-2527`, `:2601-2664`) protect current fold; no performance claim made. |
| restyle-egui-console/03 | Keep ready-for-human and blocked. Existing restyle evidence README describes only two old WASM centred-grid captures; theming evidence README records 2026-09-23 temporary-patch native captures, a cropped tall viewport and source zoom. Neither is exact-candidate ten-capture acceptance evidence. | No new captures or human review performed. |
| zoom-alignment/01 | Keep open. `Console::new` explicitly disables egui keyboard zoom (`console.rs:228-232`); existing chord test asserts Source View zoom (`kittest_tests.rs:993`), and ADR 0045 still assigns chords to source zoom (`docs/adr/0045-the-source-view-is-a-bounded-space.md:11`). | Existing source-zoom test protects old behavior, not proposed whole-UI zoom. |
| zoom-alignment/02 | Keep open behind `01`. Source zoom constants and step function remain (`console/source_view.rs:20-56`); glyph scale budget still varies with source zoom (`console/glyphs.rs:26-35`). | Many existing source-zoom tests need deliberate migration; none proves removal. |
| memory-verification/03 | Keep open. Real harness exists (`console/kittest_tests.rs:120-135`), but no retained-state/allocation steady-state comparison exists in console; no console counting allocator. Quiet-frame cursor assertions (`:904-914`) only prove no cursor movement. | Warm-up and two measured active-input spans still absent. Dependency on console-testing/04 should reflect its residual test work. |

## Independent re-verification of recent closures

| Issue | Verdict and actual evidence | Tests found |
|---|---|---|
| console-testing/03 | Closure as superseded is justified. Exact dark/light Theme keys asserted (`theme.rs:1223/:1372`), corresponding chrome records asserted (`style.rs:881/:1267`). Old 22-token/Marker acceptance no longer matches Theme architecture. | Four complete assertion bodies inspected; selected for focused execution below. |
| console-testing/04 | Prior resolved status overstates acceptance. Harness and major interactions exist, but normal Source Backspace, all-four-edge arrow clamping, and ignored Source-focused Enter/key-release integration remain absent. `kittest_tests.rs:879-914` only presses Right three times and Down once. Only Backspace at `:1821` is under focused-menu suppression; only Enter at `:2478` answers a modal. `pressed: false` entries are pointer releases. Reopen or explicitly track those residual criteria. | Prior audit also understated existing coverage: Space asserts actual Playing (`:2118-2128`); Delete updates title to unmarked after sole character erased (`:2595-2608`). No ArrowUp occurs in kittest. Typing, pointer selection, File/Quit/View, removed-menu checks are real. Dependency rationale exists in `console/Cargo.toml:175-203`. |
| console-testing/05 | Wontfix is justified for centred-fit design. ADR 0045 `:9` and ADR 0047 `:7/:15` choose top-left with margin and explicitly reject centring. Existing pure viewport geometry tests retain square/pixel/degenerate protection. | `a_viewport_with_no_area_answers_no_cell` (`grid_viewport.rs:814`), `every_cell_is_a_whole_number_of_physical_pixels` (`:829`), invalid scale cases (`:888/:913`). |
| retired-glyph-vocabulary/04 | Superseded closure justified. No ConsolePalette/PALETTE or marker/highlight palette fields remain in console source. `theme.md:199-202` keeps the retirement explanation. Other old token-count criteria belong to historical superseded palette work. | Exact-value Theme tests replace retired palette tests. |
| render-frame-derivation/02 | Wontfix justified: no classify_cursor_bloom/signal_breakup/DEFAULT_HIGHLIGHT_DOT_SPACING remains. Current effect bounds expand the cursor rectangle by fixed cell radius (`cursor_effects.rs:191-193`); `marks.rs:8` hash is seam logic. No old all-Grid bloom derivation remains to optimize. | Current cursor-effect tests exercise geometry; no claimed benchmark speedup. |

## Verification and changes

Changed only this report and theming/10's inherited-properties checkbox plus a dated evidence comment.
Parent auditor owns disposition of console-testing/04.

Focused nextest attempted with six filters: exact dark/light Source/chrome value tests,
`explicit_color_overrides_the_parent`, and the real-app arrow-key test.
Both normal and escalated runs failed before compilation because sccache returned
`Operation not permitted`. A `RUSTC_WRAPPER=` retry was started; final result below.

No source tests added. Full workspace clippy/nextest, WASM, browser suite, dependency audit,
inspection and benchmark comparisons were not run for this Markdown-only audit; whole-tier gates
and benchmarks are deferred to CI. No live UI assertion is claimed. Risk: no public API,
unsafe, dependency, feature or runtime performance changes. Documentation-only corrections.

Final focused execution: **6 passed, 485 skipped**, nextest run
`384aeb79-07a0-4b7a-830c-0c6ad94c2339`:

```sh
RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --package console --locked -E 'test(theme::tests::explicit_color_overrides_the_parent) | test(theme::tests::okabe_ito_defines_every_key_at_the_schema_values) | test(theme::tests::orcvs_light_defines_every_key_at_the_recorded_values) | test(style::tests::okabe_ito_chrome_matches_the_decided_record) | test(style::tests::orcvs_light_chrome_matches_the_decided_record) | test(console::kittest_tests::arrow_keys_move_the_cursor_through_the_source_input_path)'
```

The same command without `RUSTC_WRAPPER=` failed twice as described above.

Tracker gates: `node --test scripts/tests/roadmap.test.ts` — passed 10/10;
`node scripts/roadmap.ts > /dev/null` — passed.
