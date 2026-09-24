# 04 — Read settings from a config file

**What to build:** On native, `~/.orcvs/config.toml` supplies the dark Theme, the light Theme, Glitch amount and Glitch frequency at startup. The Settings menu and both Theme pickers leave the console. The web runs on built-in defaults.

**Blocked by:** 02 — Restructure the menu bar.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Keys: `theme.dark`, `theme.light` (Theme identities), `cursor_effects.glitch_amount`, `cursor_effects.glitch_frequency` (0–100). Every key is optional; an absent key is the built-in default.
- [ ] A missing file is not an error. An unreadable or malformed file, an unknown key, an out-of-range value, or an identity no Theme has is reported through the Theme notice channel, and the affected keys fall back to their defaults.
- [ ] A Theme identity that names a Theme of the wrong appearance is refused the same way.
- [ ] The Settings menu and the dark and light Theme pickers are removed.
- [ ] The storage keys that held Theme selections and cursor effects are no longer read or written.
- [ ] Reduced motion still governs the effective cursor effects over the configured ones.
- [ ] Web builds read no config and show no config notice.
- [ ] Tests read config from an isolated directory, never the user's home.
- [ ] Scoped gates for `console` pass, plus the no-default-features persistence arm.
