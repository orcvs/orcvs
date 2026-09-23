# 16 — Round-trip the theme preference through storage

**What to build:** A test that saves egui memory holding a non-default `ThemePreference` through the storage codec the shipped binary uses, restores it into a fresh context, constructs the console, and finds the preference intact. Residual of `02`.

**Blocked by:** None — can start immediately.

**Status:** needs-triage

- [ ] Under `persistence`, the test writes egui memory with a `Light` (or `Dark`) preference through `RonFileStorage` in an `IsolatedRonDir` (`console/src/persistence.rs`), reads it back into a new `egui::Context`, runs `Console::new`, and asserts the preference is unchanged.
- [ ] The test builds its input below the shipped entry point, without a seam in shipped code.
- [ ] `cargo nextest run --package console --locked` and the `--no-default-features` arm pass.

## Comments

**2026-09-23 — opened from a CodeRabbit review of the audit.** `02`'s "keeps it across a restart" line was ticked on `console_new_keeps_a_theme_preference_already_on_the_context` (`console/src/console.rs:3265`), which sets the preference on the context before `Console::new`. That proves `Console::new` does not overwrite a restored preference, but not that the preference survives eframe's save and restore of egui memory. `02` records the same limit and now leaves that line unticked, pointing here.
