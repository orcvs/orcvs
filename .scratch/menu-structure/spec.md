# Menu structure

The console's menus for the first release: what each menu holds, what leaves the menus entirely, and the Source File that Open and Save read and write.

## Why

The top bar today mixes three kinds of thing. File holds `Load Function reference` and Quit. View holds Diagnostics beside the appearance mode radios and two Theme pickers. Settings holds the two Glitch sliders. Notices sit loose in the bar. There is no way to open or save a Source; the only persistence is the autosave in eframe storage.

## Target

```
Native                          Web (release)
File                            File
  New           ⌘N                New
  Open…         ⌘O              View
  Save          ⌘S                Zoom In / Zoom Out / Reset Zoom
  Save As…      ⇧⌘S               ─────
  ─────                           Diagnostics
  Quit          ⌘Q              Help
View                              Function Reference
  Zoom In       ⌘=
  Zoom Out      ⌘−
  Reset Zoom    ⌘0
  ─────
  Diagnostics
Help
  Function Reference
                     [notices] [💻 🌙 ☀]
```

- Menu items show their shortcut with egui's `Button::shortcut_text(ctx.format_shortcut(..))`. The web hides the labels for chords the browser reserves (⌘N, ⌘Q, ⌘W).
- The zoom items trigger the same actions the existing chords already do.
- Notices are status, not menus: they stay in the top bar, pushed right, beside the mode control.

## Decisions

**The Grid is always 256 by 256** ([ADR 0054](../../docs/adr/0054-a-grid-is-always-256-by-256.md)). A Source File therefore carries no dimensions.

**Source File format.** Plain text, one line per row, one character per Cell, a space for an empty Cell, extension `.orcvs`. Read LF or CRLF; write LF, trimming trailing spaces and trailing empty lines. A file is refused whole — with a notice naming line and column, and the running Source untouched — when it holds a byte that is not printable ASCII or space (tabs included), more than 256 lines, or a line longer than 256 characters. Only the Source is saved; Bpm and the other running opts are not.

**Settings are config-file only.** `~/.orcvs/config.toml`, beside `~/.orcvs/themes/`, holds `theme.dark`, `theme.light`, `cursor_effects.glitch_amount` and `cursor_effects.glitch_frequency`. Read at startup; errors reach the same notice channel as Theme files. No hot reload, no settings UI. The Settings menu and both Theme pickers leave the console, and the storage keys that held those values are no longer read.

**Appearance mode is a three-state control, not a menu.** 💻 Follow the OS / 🌙 Dark / ☀ Light as icon-only selectable buttons with hover text, right-aligned in the top bar. It sets egui's own `ThemePreference`, which eframe already persists, so the mode has no config key and one owner. A two-state switch (`egui::widgets::global_theme_preference_switch`) was rejected: it cannot return to Follow the OS, the default and the native behaviour. egui's own `global_theme_preference_buttons` carries text labels; ours is the same control without them.

**A document model on native.** The console tracks the open Source File's path and whether the Source has changed since it was opened or saved. The window title shows the file name and an unsaved marker. New, Open, Help > Function Reference, Quit and closing the window ask before discarding unsaved changes. Autosave stays as session recovery; Save writes the file.

**Web file I/O is deferred.** `rfd` reads a picked file on wasm32, but its `save_file` and `FileHandle::write` are unsupported there (0.17.2), and a download through `web-sys` or the Chromium-only File System Access API is its own work. Open without Save is half a feature, so the web gets neither, and no config file. It keeps autosave, built-in defaults, and Theme import by drag and drop.

**Native dialogs use `rfd`,** synchronous API, added through `rust-dependency-change` with its rationale recorded.

## Out of scope

- Native OS menus (`muda`); egui's in-window menu bar stays.
- Recent files.
- Config hot reload and any settings UI.
- A read-only Function Reference window; for release it is a guarded Open.
- Web Open, Save and config.

## Relation to other efforts

- `file-new/` supplies File > New and the first confirmation. ADR 0054 removes its Grid-drift motivation; `02` and `03` still stand as the empty-Source Open and the confirmation pattern `06` here generalises.
- `theming/` owns Theme loading; `04` here moves only the selection out of the UI.
