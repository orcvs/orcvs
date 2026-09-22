# 09 — Move the Cursor Effect's motion into settings

**What to build:** Glitch amount and Glitch frequency become console settings, independent of the Theme, beside the operating system's reduced-motion preference. After `06` moves the Cursor Effect's colours into the Theme, they are the only part of `CursorEffectSettings` left.

**Blocked by:** 06 — Paint the Source from a named Theme.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Glitch amount and Glitch frequency live in the console's settings, with today's defaults (60 and 55), and no Theme carries either.
- [ ] Changing the dark or light Theme, or the mode, never changes either value.
- [ ] Their controls move out of the `Theme` menu. They describe motion, not appearance.
- [ ] With `persistence`, they are restored across restart. An absent or malformed stored value falls back to the defaults. The `cursor_effects` key is either reduced to these two values or left unread in favour of the settings key. The choice is recorded here.
- [ ] Amount zero paints one clear, stationary Cursor frame without decorative noise and schedules no cursor-effect repaints, even when the stored frequency is positive. The scheduling change is intentional: today's `repaint_after` checks frequency alone.
- [ ] Frequency zero freezes the current effect, which may retain decorative fragmentation when amount is nonzero, and schedules no cursor-effect repaints. It does not implicitly set amount to zero or clear the effect.
- [ ] Reduced motion paints a clear, stationary frame and schedules no cursor-effect repaints. It affects the effective motion settings, without overwriting the viewer's stored amount or frequency.
- [ ] Stopping cursor-effect repaints does not suppress independent Playback, Run Clock or input-driven repaints.
- [ ] Focused tests cover amount zero with positive frequency, frequency zero with nonzero amount, reduced motion with retained stored preferences, and continued Playback/Run Clock repaint scheduling when the cursor effect has no deadline.
- [ ] `cargo nextest run --package console --locked` and the `--no-default-features` arm both pass.

## Comments

**2026-09-21 — opened from `01`.** ADR 0053 decides that a Theme sets how things look, never how much they move. A viewer who needs the glitch off should not have to make a custom Theme for every Theme they use. VS Code's `editor.cursorBlinking` and Zed's `cursor_blink` are settings for the same reason.

**2026-09-21 — zero settings clarified by the user.** Amount zero gives a clear, stationary frame; frequency zero freezes the current effect and may retain fragmentation; reduced motion gives a clear, stationary frame. The user also confirmed stopping unnecessary cursor-effect repaints at amount zero while preserving Playback's independent repaints. This replaces the earlier acceptance that conflated the two zero values and described all scheduling as unchanged.

**2026-09-22 — explicit role backgrounds confirmed.** Role backgrounds are explicit colour properties, such as `source.function.background`, `source.number.background`, `diagnostic.background` and `output_portal.background`. They composite over the uniform `cell.background` under the confirmed fixed precedence. Changing a role foreground does not recalculate its background. There is no `fill_tint` property or shared tint-strength setting in the Orcvs Theme format. Extract and verify the built-in role background colours from current rendering so its appearance is preserved; illustrative colours in the interview are not accepted defaults.
