# 02 — Record that the Overview floats over the window's corner

**What to build:** ADR 0081, "The Overview floats over the window's corner", amending ADR 0080. ADR 0080 places the Overview docked beside the Source View and rejects a floating overlay because it covers Cells the performer is working on. The Overview is now an egui `Area` pinned so its picture's corner is the window's bottom-right corner, over the Source, the bottom panel and docked Diagnostics, because a docked panel took 260 points or more of width from the Source. The ADR records that trade, that the picture fits in half the window's width and the height below the menu bar, and that its layer is the picture's rect so it takes no pointer from the Source around it. ADR 0080's status line names the amendment, and `docs/adr/README.md` lists 0081. `overview.rs`'s module doc stops saying "docked at the right".

**Blocked by:** none

**Status:** resolved

- [x] `docs/adr/0081-the-overview-floats-over-the-windows-corner.md` exists in the repository's ADR format and amends ADR 0080.
- [x] ADR 0080's status line and `docs/adr/README.md` reference it.
- [x] No module doc or comment in `console` calls the Overview docked.
- [x] `RUSTDOCFLAGS="-D warnings" cargo doc --package console --no-deps --document-private-items --locked` passes.

## Comments

- 2026-10-24: `docs/adr/README.md` documents numbering and keeps no list of ADRs, so 0081 is referenced from ADR 0080's status line and cited in `overview.rs`'s module doc rather than listed there. `CONTEXT.md`'s Overview entry says it floats over the window's corner in place of "beside the Source View".
