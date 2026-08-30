# 15 — Render a Cell that has no glyph

**What to build:** Show the character a Cell holds even when the Cell carries no glyph classification. Typing a single character into an empty Source stores it correctly and then renders nothing, because a one-character run parses to no Expression and so receives no glyph, and the render path drops the content whenever the glyph is absent.

**Blocked by:** None (can start immediately).

**Status:** resolved

- [x] A Cell holding a character renders that character; Source gives otherwise-unclassified occupied Cells `Glyph::Char` before publication.
- [x] A Cell holding no character continues to render the background: marker, highlight, or space.
- [x] Typing one character and then a second, completing a Function, shows the first character throughout — it never disappears and reappears.
- [x] Superseded by the stronger invariant: the renderer-facing interface cannot observe a Cell with content and no glyph; Source and App tests cover the formerly failing lone-character state.

## Notes

Found while implementing ticket 09, not yet triaged by a human.

The Source is right: an edit reports the Cell with `content: Some('+')` and `glyph: None`, and an existing Source test asserts exactly that. The loss is in the render path, which treats an absent glyph as "nothing here" and falls through to the background, discarding the content it was handed.

Reproduces at any Source size and in any Cell; it is not a row-edge case. A lone `+`, a lone digit, and the first character of any Function you are part-way through typing are all invisible.

## Answer

Superseded by Issue 11. Source now assigns `Glyph::Char` to every occupied Cell that has no parser classification before publishing an accepted revision. `SourceCommander::get` reads content and Glyph under the same lock, so the renderer-facing `App::get` seam cannot observe content without a Glyph. Existing Source and App regressions cover lone-character visibility and reclassification when a Function is completed.
