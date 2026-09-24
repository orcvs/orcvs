# 04 — Read settings from a config file

**What to build:** On native, `~/.orcvs/config.toml` supplies the dark Theme, the light Theme, Glitch amount and Glitch frequency at startup. The Settings menu and both Theme pickers leave the console. The web runs on built-in defaults.

**Blocked by:** 02 — Restructure the menu bar.

**Status:** resolved

**Tags:** release/v1

- [x] Keys: `theme.dark`, `theme.light` (Theme identities), `cursor_effects.glitch_amount`, `cursor_effects.glitch_frequency` (0–100). Every key is optional; an absent key is the built-in default.
- [x] A missing file is not an error. An unreadable or malformed file, an unknown key, an out-of-range value, or an identity no Theme has is reported through the Theme notice channel, and the affected keys fall back to their defaults.
- [x] A Theme identity that names a Theme of the wrong appearance is refused the same way.
- [x] The Settings menu and the dark and light Theme pickers are removed.
- [x] The storage keys that held Theme selections and cursor effects are no longer read or written.
- [x] Reduced motion still governs the effective cursor effects over the configured ones.
- [x] Web builds read no config and show no config notice.
- [x] Tests read config from an isolated directory, never the user's home.
- [x] Scoped gates for `console` pass, plus the no-default-features persistence arm.

## Comments

- Schema as implemented (`console/src/config.rs`): top-level tables `[theme]` (`dark`, `light`:
  non-empty strings) and `[cursor_effects]` (`glitch_amount`, `glitch_frequency`: integers 0–100).
  Anything else is an unknown key. Each problem is its own notice and falls back only the key it
  names; a file that is not TOML, not UTF-8, larger than 64 KiB or unreadable falls back whole.
- No dependency was added: `toml` and `serde` were already shipped for Theme documents.
- Identities are checked where Theme selections already were: `SelectedThemes::new` resolves each
  against the registry and reports a missing, refused, conflicted or wrong-appearance Theme naming
  `theme.dark` / `theme.light`, presenting the appearance's built-in. Config notices join the
  registry's notices, so both arrive under the top bar's notices menu, titled Notices because it
  carries settings problems as well as Theme ones.
- `Config::start` is the shipped entry: `~/.orcvs/config.toml` on native, the defaults on web.
  Tests call `Config::read(path)` on a temporary directory, or build a `Config` value directly.
- ADR 0053 still describes Theme pickers in its Consequences; it is an accepted record and was left
  as written. `console/src/theme.md` describes the config file.
- Resolved after review: on the web nothing could select an imported Theme. The pickers and the
  stored selection keys are gone and the web reads no config file, so the selection is always the
  two built-ins, and a dropped Theme file was imported and stored but never presented. Web Theme
  import is disabled for v1: the drop reader, the import path, the `imported_themes` storage keys
  and the View menu's drop hint are removed, and the web has the built-ins alone. Values earlier
  builds stored under `imported_themes` are no longer read.
- Open question from review (not changed here): values earlier builds stored under `dark_theme`,
  `light_theme` and `cursor_effects` are ignored without a notice, as this issue asks. An upgrading
  viewer who had picked a custom Theme is not told to move it to `config.toml`.
