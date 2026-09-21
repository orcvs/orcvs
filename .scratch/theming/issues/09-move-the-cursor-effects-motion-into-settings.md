# 09 — Move the Cursor Effect's motion into settings

**What to build:** Glitch amount and Glitch frequency become console settings, independent of the Theme, beside the operating system's reduced-motion preference. After `06` moves the Cursor Effect's colours into the Theme, they are the only part of `CursorEffectSettings` left.

**Blocked by:** 06 — Paint the Source from a scheme.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Glitch amount and Glitch frequency live in the console's settings, with today's defaults (60 and 55), and no Theme carries either.
- [ ] Changing the dark or light Theme, or the mode, never changes either value.
- [ ] Their controls move out of the `Theme` menu. They describe motion, not appearance.
- [ ] With `persistence`, they are restored across restart. An absent or malformed stored value falls back to the defaults. The `cursor_effects` key is either reduced to these two values or left unread in favour of the settings key. The choice is recorded here.
- [ ] Amount zero and frequency zero keep today's meaning: one clear frame without decorative noise, and no scheduled repaints.
- [ ] `cargo nextest run --package console --locked` and the `--no-default-features` arm both pass.

## Comments

**2026-09-21 — opened from `01`.** ADR 0053 decides that a Theme sets how things look, never how much they move. A viewer who needs the glitch off should not have to make a custom Theme for every Theme they use. VS Code's `editor.cursorBlinking` and Zed's `cursor_blink` are settings for the same reason.
