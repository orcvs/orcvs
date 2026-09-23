# 02 — Keep the restored theme preference at startup

**What to build:** Stop the console from overwriting a persisted theme preference at startup, and
make sure a light preference cannot produce a half-styled console.

**Blocked by:** None — can start immediately.

**Status:** resolved

**Tags:** release/v1

- [x] `Console::new` no longer calls `ctx.set_theme(egui::Theme::Dark)`.
- [x] `Console::new` registers the current style for both `Theme::Dark` and `Theme::Light`. One
      palette still exists, so both registrations use it.
- [x] The console reinstalls its style on every launch. eframe restores egui memory but skips the
      styles, so the application owns them.
- [ ] *(Storage round-trip test: `16`.)* A `persistence` build restores the stored `ThemePreference` and keeps it across a restart.
- [x] A build without `persistence` starts from `System`.
- [x] One owner holds the preference: either egui memory, or a later Orcvs settings object. Never
      both.
- [x] `mise run test_persistence` passes.

## Comments

**The defect.** eframe restores egui memory before it calls the application creator, on native and on
web. `ThemePreference` is part of the serializable `Options` that eframe restores. `Console::new`
then calls `set_theme(Theme::Dark)` with no condition, which replaces the restored value. The `console`
crate already has a `persistence` feature that turns on `eframe/persistence`, so the defect is live.

**Why the second acceptance line is required.** Removing the `set_theme` call alone is not safe
today. `Console::new` registers a style for the dark theme only. If the restored preference resolves
to Light, egui uses its own default light style for the menus and windows, while the Source Grid
still paints from the dark `PALETTE`. The result is a mixed console. Registering the one palette for
both themes removes the preference override without introducing that state.

**Relationship to 03.** `03` replaces the duplicate registration with `install(ctx)` and a real
per-theme palette. This issue is the smaller, independent step: it fixes persistence now, and does
not wait for the palette decision in `restyle-egui-console/02`. `03` therefore lists this issue
as a blocker so the two changes do not race in the same function.

`docs/research/egui-theming.md` on `feat/egui-theming` records this analysis. `03` recovers that
document.

**2026-09-21 — ADR 0053 and the "one owner" line.** Under ADR 0053 the settings hold a dark Theme, a light Theme, and a mode. egui's `ThemePreference` (System, Dark, Light) is exactly that mode, so it may stay the one owner of the mode. The two Theme names are Orcvs settings `06` adds. This issue's scope is unchanged. It still registers the one existing style for both themes, and `03` replaces that with a style per Theme.

**2026-09-22 — shared presentation retained through `03`.** The earlier comments assigning distinct registration to `03` are superseded. `03` derives the shared style from the resolved dark Theme but keeps both egui appearance slots on that presentation; `04` supplies and accepts the light definition before distinct registration and switching are enabled.

**2026-09-23 — resolved by PR #119; recorded here by the 2026-09-23 audit of the merged pull requests against their issues.** #119 merged into `theme-paint-source` and reached `main` through #120's merge `c7d49156`. Each line checked against `main`: `Console::new` calls only `install` (`console/src/console.rs:812-827`), and every remaining `set_theme` call is in tests. `install` shares one `Arc<Style>` between both slots (`console/src/style.rs:761-765`), tested by `install_shares_one_style_between_both_theme_slots`. `console_new_keeps_a_theme_preference_already_on_the_context` and `console_new_leaves_a_fresh_context_on_the_system_preference` cover the two builds. `persistence.rs` stores only the dark and light Theme identities, so egui memory is the preference's one owner. `mise run test_persistence` ran inside `check_merge` on the push of `c7d49156` to `main`, which passed. Limit: the restart is simulated by setting the preference on the context before `Console::new`; no test round-trips egui memory through real storage, so that line stays unticked and the test is `16`.
