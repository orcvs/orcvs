# 01 — Open the Overview and the Diagnostics from settings

**What to build:** ADR 0091's `[overview] open` and `[diagnostics] open`. Each is a boolean in
`config.toml` saying whether that view is open when the console starts. Absent, the Overview is
open and the Diagnostics closed, as today. A mistyped value is a notice and falls back alone.

See `../spec.md`.

**Blocked by:** None

**Status:** resolved

- [x] The settings reader reads both keys, and a mistyped value falls back alone with a notice.
- [x] A console started with either key set starts with that view open or closed.
- [x] The settings documentation shows both sections.

## Comments

- `Config` holds `overview_open` and `diagnostics_open`, and `Console::new` starts each view from them.
- `the_overview_and_the_diagnostics_open_as_the_file_says` reads both keys and a mistyped value of each.
- `a_console_starts_with_the_overview_and_the_diagnostics_its_settings_open` starts a console with the
  Overview closed and the Diagnostics open, and opens the Overview from the View menu.
