# 02 — Restructure the menu bar

**What to build:** File, View and Help hold what `spec.md` lists. View gains Zoom In, Zoom Out and Reset Zoom above Diagnostics. `Load Function reference` leaves File for Help as `Function Reference`. Notices move to the right of the bar.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] View holds Zoom In, Zoom Out, Reset Zoom, a separator, then Diagnostics. Each zoom item triggers the same action as its chord and shows that chord as shortcut text.
- [ ] Help holds Function Reference, which does what `Load Function reference` does today.
- [ ] File holds only what is built so far (New once `file-new/02` lands, Quit on native).
- [ ] The persistence notice and Theme notices sit right-aligned in the top bar.
- [ ] The top-bar test asserts the new menu titles and the absence of any Function reference item in File.
- [ ] Scoped gates for `console` pass, and the egui skill's guidance is followed.
