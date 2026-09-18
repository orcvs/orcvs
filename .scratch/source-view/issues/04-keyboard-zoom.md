# 04: Keyboard Zoom

**What to build:** The viewer Zooms from the keyboard alone. Command `=` (or `+`) and command `-` step the Cell size by an eighth of the Source's own Cell — two points — between 0.25 and 2.0; command `0` returns to 1.0. Command is ⌘ on macOS and Ctrl elsewhere. See ADR 0045.

**Blocked by:** 02

**Status:** resolved

- [x] Command `=` and command `+` Zoom in one step; command `-` Zooms out one step; command `0` returns to Zoom 1.0.
- [x] Zoom stops at 0.25 and at 2.0.
- [x] Bare `+`, `-`, `=` and `0` still reach the Source as Cell input, and a command chord never does.
- [x] A Zoom that would open a gap past an edge settles the Source View back inside the Grid.
- [x] The Glyph is laid out at the Cell size of each step.
