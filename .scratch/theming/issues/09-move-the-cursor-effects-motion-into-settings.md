# 09 — Move the Cursor Effect's motion into settings

**What to build:** Glitch amount and Glitch frequency become console settings, independent of the Theme, beside the operating system's reduced-motion preference. After `06` moves the Cursor Effect's colours into the Theme, they are the only part of `CursorEffectSettings` left.

**Blocked by:** 06 — Paint the Source from a named Theme.

**Status:** ready-for-agent

**Tags:** release/v1

- [x] Glitch amount and Glitch frequency live in the console's settings, with today's defaults (60 and 55), and no Theme carries either.
- [x] Changing the dark or light Theme, or the mode, never changes either value.
- [x] Their controls move out of the `Theme` menu. They describe motion, not appearance.
- [x] With `persistence`, they are restored across restart. An absent or malformed stored value falls back to the defaults. The `cursor_effects` key is either reduced to these two values or left unread in favour of the settings key. The choice is recorded here.
- [ ] Amount zero paints one clear, stationary Cursor frame without decorative noise and schedules no cursor-effect repaints, even when the stored frequency is positive. The scheduling change is intentional: today's `repaint_after` checks frequency alone.
- [ ] Frequency zero freezes the current effect, which may retain decorative fragmentation when amount is nonzero, and schedules no cursor-effect repaints. It does not implicitly set amount to zero or clear the effect.
- [ ] Reduced motion paints a clear, stationary frame and schedules no cursor-effect repaints. It affects the effective motion settings, without overwriting the viewer's stored amount or frequency.
- [ ] Stopping cursor-effect repaints does not suppress independent Playback, Run Clock or input-driven repaints.
- [ ] Focused tests cover amount zero with positive frequency, frequency zero with nonzero amount, reduced motion with retained stored preferences, and continued Playback/Run Clock repaint scheduling when the cursor effect has no deadline.
- [x] `cargo nextest run --package console --locked` and the `--no-default-features` arm both pass.

## Comments

**2026-09-21 — opened from `01`.** ADR 0053 decides that a Theme sets how things look, never how much they move. A viewer who needs the glitch off should not have to make a custom Theme for every Theme they use. VS Code's `editor.cursorBlinking` and Zed's `cursor_blink` are settings for the same reason.

**2026-09-21 — zero settings clarified by the user.** Amount zero gives a clear, stationary frame; frequency zero freezes the current effect and may retain fragmentation; reduced motion gives a clear, stationary frame. The user also confirmed stopping unnecessary cursor-effect repaints at amount zero while preserving Playback's independent repaints. This replaces the earlier acceptance that conflated the two zero values and described all scheduling as unchanged.

**2026-09-22 — explicit role backgrounds confirmed.** Role backgrounds are explicit colour properties, such as `source.function.background`, `source.number.background`, `diagnostic.background` and `output_portal.background`. They composite over the uniform `cell.background` under the confirmed fixed precedence. Changing a role foreground does not recalculate its background. There is no `fill_tint` property or shared tint-strength setting in the Orcvs Theme format. Extract and verify the built-in role background colours from current rendering so its appearance is preserved; illustrative colours in the interview are not accepted defaults.

**2026-09-22 — Settings-menu slice implemented and reviewed, after verifying `06`.** `06` is only partly implemented on this branch (its own Status line correctly still reads `ready-for-agent`): there is no `Scheme`/named-property-template model beyond the one `okabe_ito()` built-in, and no dark/light pickers. What `09` depended on is landed: the Cursor Effect's colours are on the resolved `Theme` (`console/src/theme.rs` carries `cursor_border`, `cursor_area`, `region_background`, `cursor_background`, `region_cursor_background` and the two border widths, and `console.rs`'s `show_source`/`show_source_scene` read them from `self.theme`), and `CursorEffectSettings` (`console/src/cursor_effects.rs`) was already reduced to `amount`/`frequency`, with `encode`/`decode` packing only those two fields. What this pass did, against the first four acceptance lines above:

- Moved the "Glitch amount"/"Glitch frequency" `Slider`s out of the `Theme` menu into a `Settings` menu in the top bar (`console.rs::ui`), and removed the `Theme` menu entirely rather than leave an empty one a viewer could open — `03`/`04`/`06` add it back once there is a picker to put in it.
- Reordered `Console`'s fields so `cursor_effects`/`cursor_effect_animation` sit directly beside `reduced_motion` in the struct.
- **Persistence key decision: reduce `cursor_effects`, don't add a settings key.** `CURSOR_EFFECTS_KEY = "cursor_effects"` already only round-trips `amount;frequency`, and a value written before `06` (which carried colour groups with commas) is refused whole and falls back to the defaults — this crate's ordinary "malformed value falls back to the default" rule, not a migration. A separate settings key would have split one small value across two persisted homes for no reason. Documented at the key's own definition in `console/src/persistence.rs`.
- Corrected `console/src/theme.md`'s self-contradictions about menu contents. Left the rest of that section's larger `06`-era staleness alone, since `03`/`08` also edit `theme.md`.
- Acceptance line 2 (Theme/mode never changes Cursor effects) holds structurally: `Theme` declares no amount/frequency/motion field, and `Console::cursor_effects` is a field nothing in Theme resolution reads or writes. A behavioural test would be tautological today — nothing yet resolves `dark_theme`/`light_theme` into `self.theme` — and belongs with `06`/`07`'s pickers.
- Added `console::kittest_tests::no_menu_offers_a_reset_to_theme_defaults_button` (loops the surviving menus) and `console::kittest_tests::glitch_controls_are_offered_only_by_the_settings_menu` (absent before any menu opens, present once Settings does), renamed from `the_theme_menu_offers_no_reset_button`/`glitch_controls_live_in_the_settings_menu_not_the_theme_menu`, which depended on opening a `Theme` menu that no longer exists. Also renamed `a_focused_theme_menu_value_box_…` to `a_focused_settings_menu_value_box_…`, and added a negative assertion to `console::tests::the_bottom_panel_shows_tick_zero_and_run_clock_before_the_first_run` that "Theme" is not on the top bar. Confirmed red by temporarily reinstating `ui.menu_button("Theme", |_ui| {})` and rerunning that test before restoring the fix.
- Named the top menu bar's item gap `MENU_BAR_GAP` instead of repeating the `16.0` literal, reusing it at every call site including the one in the isolated top-panel-height test.
- Trimmed the ADR 0053 rationale that had been restated in most of the touched comments down to one short reference, at the one place it is a placement decision (`console.rs`'s `Settings` menu comment); the rest just describe what the item is now. History (why `06`/`09` moved what they moved) lives in this file's Comments, not in code.
- Defaults, zero-amount/zero-frequency semantics, and the persistence round trip (absent and malformed) were already covered by `cursor_effects.rs`'s and `persistence.rs`'s existing tests from `06`'s work; no behaviour there needed to change for this slice.
- The five remaining unticked lines above are the zero-settings clarification's own scope (opened the same day, above) — repaint scheduling at amount zero, frequency zero's retained fragmentation, reduced motion's effective-vs-stored split, Playback/Run Clock independence, and the focused tests proving all four — and are this issue's next slice of work.

`cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings`, `cargo nextest run --package console --locked` and `cargo nextest run --workspace --tests --no-default-features --locked` all pass at this point.
