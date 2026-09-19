# 01: Tab steps the Cursor by Sector

**What to build:** Tab and Shift Tab move the Cursor across the Source Grid a Sector at a time, along the Cursor's row, the way Tab steps across cells in a spreadsheet. Tab belongs to the Source while it has the keys, so egui's Tab focus navigation must not also fire and take the keys to a menu-bar button.

**Status:** resolved

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

- [x] Tab from any Cell of a Sector moves the Cursor to the first Cell of the next Sector on the same row.
- [x] Tab in the last Sector of a row leaves the Cursor in place.
- [x] Shift Tab from inside a Sector moves to that Sector's first Cell; from its first Cell, to the previous Sector's first Cell; in the first Sector's first Cell, nowhere.
- [x] Tab and Shift Tab collapse a multi-Cell Region to the Cursor.
- [x] With the Source holding the keys, Tab focuses no menu-bar widget, and the next typed character writes the Source.
- [x] With a control focused (the BPM field), Tab keeps egui's focus navigation and does not move the Cursor.
- [x] A Grid whose width is not a multiple of the Sector Seam spacing: Tab reaches the cut-short last Sector and stops there.

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

## Comments

- Implemented `Grid::next_sector`/`Grid::previous_sector` beside `left`/`right` in `orcvs/src/grid.rs`, taking `SectorSeamSpacing`; wired into `Orcvs::event_handler` via new `InputKey::Tab`/`InputKey::ShiftTab`; and mapped in `console`'s `translate_event`.
- The plain `Memory::move_focus(FocusDirection::None)` cancellation this issue's Notes describe has to run *before* `Console::ui` shows the top menu bar, not in the existing event-routing block after it — `interested_in_focus` (`egui-0.36.1/src/memory/mod.rs`) hands Tab's focus to the first focusable widget the instant it is shown, and cancelling afterward is too late to stop it. Moved the cancellation to the top of `Console::ui`.
- That unconditional placement broke the pre-existing `a_focused_theme_menu_value_box_keeps_region_and_clipboard_commands_from_the_source` kittest, which reaches a Theme menu value box by Tab alone (a pointer click there closes the menu first). Opening that menu does not itself take focus, so the cancellation was starving it forever. Guarded the cancellation with `!egui::Popup::is_any_open(&ctx)` in addition to `!self.keyboard_elsewhere`, so Tab is only intercepted for the Source while no menu is already open; once a menu is open, egui's own Tab focus order is left to reach it as before. All 208 pre-existing and new `console` tests pass with this guard in place.
- No CONTEXT.md or user-facing keybindings doc changes: no existing doc lists arrow keys or Region-collapse keybindings for Tab/Shift Tab to join.

### Review follow-up: an open menu owns Tab, not just the focus cancellation

- Code review (confirmed by two reviewers against egui 0.36.1 source) found the first fix above still let one Tab press do two jobs. With a menu open and nothing yet focused, `keyboard_elsewhere` is false (a menu button's click does not take focus) and `Popup::is_any_open` is true: the focus cancellation at the top of `Console::ui` correctly skipped, letting egui's `FocusDirection::Next` move focus — but the event-routing block further down only checked `!self.keyboard_elsewhere`, so it *also* forwarded the same Tab to `Orcvs::event_handler` and stepped the Cursor a Sector. Each site was asking "is a menu open" independently and could disagree about which Tab was whose.
- Fixed by reading `Popup::is_any_open` exactly once per frame, in a `menu_open` local at the top of `Console::ui`, and reusing that one answer at both the cancellation and the event routing (the routing now filters `InputKey::Tab`/`InputKey::ShiftTab` out of the events handed to `Orcvs::event_handler` whenever `menu_open`). The two sites can no longer read a different answer for the same frame.
- Test-first: added `console::kittest_tests::tab_with_a_menu_open_moves_focus_and_not_the_cursor`, which opens the Theme menu the same way the existing value-box test does and asserts a single Tab press does not move the Cursor. Confirmed it failed against the pre-fix code (Cursor moved from `(0, 0)` to `(8, 0)`) before applying the fix above.
- Also found and fixed: the Tab arm in `translate_event` matched a `Key::Tab` press under *any* modifiers and branched only on `modifiers.shift`, so Ctrl+Tab, Command+Tab, and Alt+Tab all reached the Source as a plain Tab or Shift Tab. Guarded the arm with `modifiers.is_none() || modifiers.shift_only()` — the same two tests `Memory::begin_pass` itself uses to decide `FocusDirection::Next`/`Previous` — so a modified Tab now falls through to `None` instead. Test-first: extended `toolkit_events_translate_only_the_input_orcvs_handles` with cases for `COMMAND`, `CTRL`, `ALT`, `COMMAND | SHIFT`, and `ALT | SHIFT` held with Tab, confirmed they failed (Command+Tab translated to `Some(KeyPressed(Tab))`) before the guard was added.
- Minor readability fix: `Grid::previous_sector`'s branch condition was `pos.x > sector_start || sector == 0`, which named the "stay at this Sector's start" branch rather than the "step back" branch its comment described. Rewritten as `pos.x == sector_start && sector > 0` (equivalent, since `pos.x >= sector_start` always holds) guarding the step-back branch instead, matching the comment's own order.
- Recorded the open-menu carve-out in `docs/adr/0048-the-source-takes-the-keys-no-control-holds.md`'s Consequences section, since it is a second, narrower exception to "no control holds the keys" alongside the one the ADR already describes for a focused control.
