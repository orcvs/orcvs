# 01: Tab steps the Cursor by Sector

**What to build:** Tab and Shift Tab move the Cursor across the Source Grid a Sector at a time, along the Cursor's row, the way Tab steps across cells in a spreadsheet. Tab belongs to the Source while it has the keys, so egui's Tab focus navigation must not also fire and take the keys to a menu-bar button.

**Status:** ready-for-agent

## Decisions

- Tab moves the Cursor to the first Cell of the next Sector on the same row.
- In the Grid's last Sector on a row, Tab leaves the Cursor where it is. There is no wrap to the next row of Sectors: the Grid clamps every other Cursor move at its edges (`Grid::right`, `orcvs/src/grid.rs:338`), and Excel does not wrap Tab at the sheet's last column either.
- Shift Tab moves the Cursor to the first Cell of the Sector it is in. If the Cursor is already on that first Cell, Shift Tab moves it to the first Cell of the previous Sector. At the Grid's first Sector it stays where it is.
- Tab and Shift Tab collapse the Region to the Cursor, as a plain arrow does.
- Tab inside a Region does not cycle through the Region's Sectors (Excel's wrap-within-selection). Left out so Tab means the same thing whatever the Region spans.

## Notes

- egui turns an unmodified Tab into `FocusDirection::Next` and Shift Tab into `FocusDirection::Previous` in `Memory::begin_pass`, before the app's `ui` runs (`egui-0.36.1/src/memory/mod.rs:596-597`). When the Source takes a Tab, cancel it with `Memory::move_focus(FocusDirection::None)` (`memory/mod.rs:934-936`) so no widget also takes focus.
- The Sector size is `SectorSeamSpacing` (default `DEFAULT_SECTOR_SEAM_SPACING`, 8), not a constant.

## Acceptance

- [ ] Tab from any Cell of a Sector moves the Cursor to the first Cell of the next Sector on the same row.
- [ ] Tab in the last Sector of a row leaves the Cursor in place.
- [ ] Shift Tab from inside a Sector moves to that Sector's first Cell; from its first Cell, to the previous Sector's first Cell; in the first Sector's first Cell, nowhere.
- [ ] Tab and Shift Tab collapse a multi-Cell Region to the Cursor.
- [ ] With the Source holding the keys, Tab focuses no menu-bar widget, and the next typed character writes the Source.
- [ ] With a control focused (the BPM field), Tab keeps egui's focus navigation and does not move the Cursor.
- [ ] A Grid whose width is not a multiple of the Sector Seam spacing: Tab reaches the cut-short last Sector and stops there.

## Research

Recorded from the review of PR 107. The routing decision is [ADR 0048](../../../docs/adr/0048-the-source-takes-the-keys-no-control-holds.md); this section keeps the research behind it.

### How keys reach the Source

- The Source gets the keys unless a control holds egui's keyboard focus. The console asks `Context::egui_wants_keyboard_input()`, which is `Memory::focused().is_some()` (`egui-0.36.1/src/context.rs:2982-2985`). egui names this as the question for an app with its own canvas: `RawInput::events` has "no way to know if egui handles a particular event, but you can check if egui is using the keyboard with `egui_wants_keyboard_input`" (`egui-0.36.1/src/data/input/raw_input.rs:56-60`).
- Waiting for widgets to consume keys cannot work. `TextEdit` reads Copy, Cut, Paste and text without consuming them (`egui-0.36.1/src/widgets/text_edit/builder.rs:1081`), and the console reads its events before any widget runs, so `consume_key` and `consume_shortcut` (`egui-0.36.1/src/input_state/mod.rs:724-743`) act too late.
- The console latches the answer at the end of each frame (`Console::keyboard_elsewhere`) and reads last frame's value. `Memory::begin_pass` clears focus on Escape before the app's `ui` runs (`egui-0.36.1/src/memory/mod.rs:596-601`), so reading focus live would let the Escape that leaves a field also collapse the Region.
- The destination ComboBox's open list takes no focus, so it is checked separately.
- A focused egui widget already takes Space and Enter as its click (`egui-0.36.1/src/context.rs:1463-1470`). Any focused control therefore has to keep keys from the Source, not only text fields.

### The Grid does not take focus

- Considered: making the Grid's one interaction rectangle a focusable widget, taking focus on click, getting keys only while it holds focus, and keeping arrows, Tab and Escape with `Memory::set_focus_lock_filter` (`egui-0.36.1/src/memory/mod.rs:903-911`), as `TextEdit` does.
- Rejected, for three reasons. With nothing focused, at startup and after Escape, keys would go nowhere, and a live-coding instrument should always accept typing. The Grid would join the Tab order, which `.scratch/source-grid-rendering/issues/02-paint-the-source-grid.md:78-81` and the kittest module docs keep it out of. Escape's double role, collapsing the Region and clearing focus, would split.
- Tab by Sector removes the remaining case for it: Tab belongs to the Source, so the Grid needs no focus to receive it.
- A widget per Cell was never an option: ADR 0040 and the kittest module docs rule it out.
- Rules this depends on: nothing else in the console area may be focusable. The pan rectangle senses `Sense::CLICK | Sense::DRAG` rather than `Sense::click_and_drag()` for that reason, and `tab_never_focuses_the_console_area_the_source_is_shown_in` guards it.

### Spreadsheet precedent (Excel, from memory, not checked against a current build)

- Tab moves one cell right and Shift Tab one cell left. At the sheet's last column Tab does not wrap to the next row. Google Sheets and Numbers behave much the same.
- Inside a selected range, Tab walks the selection left to right, drops to the start of the next row at the range's right edge, and returns to the first cell after the last one. Not adopted: Tab would then mean something different with a Region selected than with one Cell.
- Tab then Enter returns to the column where a run of Tabs started, one row down. Not applicable: Enter is not "commit and go down" in Orcvs.
