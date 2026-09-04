# 06 — Keep the restored theme preference at startup

**What to build:** Stop the console from overwriting a persisted theme preference at startup, and
make sure a light preference cannot produce a half-styled console.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] `Console::new` no longer calls `ctx.set_theme(egui::Theme::Dark)`.
- [ ] `Console::new` registers the current style for both `Theme::Dark` and `Theme::Light`. One
      palette still exists, so both registrations use it.
- [ ] The console reinstalls its style on every launch. eframe restores egui memory but skips the
      styles, so the application owns them.
- [ ] A `persistence` build restores the stored `ThemePreference` and keeps it across a restart.
- [ ] A build without `persistence` starts from `System`.
- [ ] One owner holds the preference: either egui memory, or a later Orcvs settings object. Never
      both.
- [ ] `mise run test_persistence` passes.

## Comments

**The defect.** eframe restores egui memory before it calls the application creator, on native and on
web. `ThemePreference` is part of the serializable `Options` that eframe restores. `Console::new`
then calls `set_theme(Theme::Dark)` with no condition, which replaces the restored value. The `shell`
crate already has a `persistence` feature that turns on `eframe/persistence`, so the defect is live.

**Why the second acceptance line is required.** Removing the `set_theme` call alone is not safe
today. `Console::new` registers a style for the dark theme only. If the restored preference resolves
to Light, egui uses its own default light style for the menus and windows, while the Source Grid
still paints from the dark `PALETTE`. The result is a mixed console. Registering the one palette for
both themes removes the preference override without introducing that state.

**Relationship to 04.** `04` replaces the duplicate registration with `install(ctx)` and a real
per-theme palette. This issue is the smaller, independent step: it fixes persistence now, and does
not wait for the palette decision in `02`. `04` therefore lists this issue as a blocker so the two
changes do not race in the same function.

`docs/research/egui-theming.md` on `feat/egui-theming` records this analysis. `04` recovers that
document.
