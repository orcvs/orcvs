# 01 — Show failures as Toasts

Status: ready-for-agent
Type: task

**What to build:** every standing Panel message becomes a Toast in the window's top-right corner,
and the Panel's growing status line goes ([spec](../spec.md), ADR 0100).

- [ ] One Toast per standing message, in the Panel's existing order, stacked beneath the menu bar,
      16 points in from the top and right edges, over the Source and clear of the Minimap.
- [ ] A Toast is an `egui::Area` anchored to the corner: a popup frame outlined in the error colour,
      the output's label (`MIDI`, `UDP/OSC`, `ENGINE`) in the weak text colour, a `×` button, and
      the message in the error colour, wrapped.
- [ ] `×` removes the Toast. It does not return for the same message; a changed message, or one
      that clears and comes back, raises a new Toast.
- [ ] `show_statuses` and the status height go. The Panel's height is the same with no message and
      with every message.
- [ ] Kittests drive the shipped console: find a Toast's label and message beneath the menu bar,
      press `×` and find it gone, and compare the Panel's height with and without messages.
