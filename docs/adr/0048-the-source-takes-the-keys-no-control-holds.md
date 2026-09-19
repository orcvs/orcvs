# The Source takes the keys no control holds

Status: accepted. Extends [ADR 0046](0046-the-primary-drag-selects-a-region.md), whose Region commands this routing carries.

The console's Source Grid is not an egui widget that holds focus. It gets every key while no control holds egui's keyboard focus, and none while one does. The console asks `Context::egui_wants_keyboard_input()`, which is `Memory::focused().is_some()` (`egui-0.36.1/src/context.rs:2982-2985`), and egui names that as the question for an app with its own canvas: `RawInput::events` has "no way to know if egui handles a particular event, but you can check if egui is using the keyboard with `egui_wants_keyboard_input`" (`egui-0.36.1/src/data/input/raw_input.rs:56-60`).

**Any focused control keeps the keys, not a named list.** The console used to hold keys back for the BPM field and the destination ComboBox by name. Every other control that took focus leaked them: a Theme menu Slider's value box passed command `A`, Backspace, Cut and Paste to the Source, which then emptied the whole Grid with no undo. A focused egui widget also takes Space and Enter as its click (`egui-0.36.1/src/context.rs:1463-1470`), so a focused button double-acted with Playback. Asking egui covers controls that have not been written yet. The destination ComboBox's open list takes no focus, so it is asked for separately.

**The answer is last frame's.** The console reads its events before any widget runs, so it cannot ask this frame's widgets, and waiting for them to consume keys cannot work either: `TextEdit` reads Copy, Cut, Paste and text without consuming them (`egui-0.36.1/src/widgets/text_edit/builder.rs:1081`). The console latches the answer once every widget has run and reads it on the next frame. Asking at the start of the frame instead would be wrong for Escape: `Memory::begin_pass` clears focus on Escape before the app's `ui` runs (`egui-0.36.1/src/memory/mod.rs:596-601`), so the Escape that leaves a field would also collapse the Region.

**Keys a control took still disarm a fill.** Command Enter arms a fill for the next event. Keys that went to a control are that event even though the Source never saw them, so the console disarms the fill for them.

**Nothing in the console area takes focus.** The Grid's interaction rectangle senses `Sense::CLICK`, and the pan rectangle behind it `Sense::CLICK | Sense::DRAG`: `Sense::click()` and `Sense::click_and_drag()` add `FOCUSABLE` (`egui-0.36.1/src/sense.rs:60-83`). A focusable rectangle there would let Tab land on the area the Source is shown in, count as a control holding the keyboard, and lock the Source out until Escape or a click.

## Rejected alternatives

**The Grid holds focus.** The Grid's rectangle could be a focusable widget, as `TextEdit` is: take focus on click, get keys only while it holds focus, and keep arrows, Tab and Escape with `Memory::set_focus_lock_filter` (`egui-0.36.1/src/memory/mod.rs:903-911`). It makes focus the one rule and needs no latch. But with nothing focused, at startup and after Escape, keys would go nowhere, and a live-coding instrument should always accept typing without a click first. The Grid would join the Tab order, which the painted Grid was built to stay out of. And Escape, which today both collapses the Region and leaves a control, would have to pick one.

**A widget per Cell.** Never an option: [ADR 0040](0040-the-console-paints-from-a-value.md) paints the Grid as one rectangle, and a widget per Cell would mint a thousand AccessKit nodes.

**Widgets consume the keys they use.** egui has `consume_key` and `consume_shortcut` (`egui-0.36.1/src/input_state/mod.rs:724-743`), but the console reads its events before any widget runs, and `TextEdit` does not consume what it reads.

**Name each control that holds keys back.** What the console did. Every control added later leaks until someone notices, and the leak this ADR records was found in review, not by a test.

## Consequences

Tab belongs to the Source while it has the keys: `.scratch/sector-navigation/issues/01-tab-steps-the-cursor-by-sector.md` gives it the move to the next Sector and cancels egui's Tab focus navigation with `Memory::move_focus`. Until that lands, an unfocused Tab moves egui's focus to the menu bar, and the Source gets no keys until Escape or a click on the Grid. `tab_never_focuses_the_console_area_the_source_is_shown_in` guards the console area staying out of the Tab order.

An open menu is a further carve-out from "no control holds the keys" and holds Tab specifically, without holding egui's keyboard focus the way the rest of this ADR reads it: opening one by a pointer click takes no focus, so `keyboard_elsewhere` alone cannot tell the console a menu is open. Tab is how a viewer already in one reaches a control inside it — `a_focused_theme_menu_value_box_keeps_region_and_clipboard_commands_from_the_source` reaches its value box only that way, because a click there closes the menu first — so while a menu is open, Tab is egui's own focus navigation and never also the Source's move to the next Sector. `Console::ui` reads `Popup::is_any_open` once per frame and reuses that one answer at both the focus cancellation and the event routing, so the two cannot disagree about which Tab reaches which destination; asking twice once let a menu-opening frame answer "open" at one site and "not yet focused" at the other, and the same press did both jobs. `tab_with_a_menu_open_moves_focus_and_not_the_cursor` guards it.
