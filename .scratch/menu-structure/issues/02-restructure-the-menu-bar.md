# 02 — Restructure the menu bar

**What to build:** File, View and Help hold what `spec.md` lists. View gains Zoom In, Zoom Out and Reset Zoom above Diagnostics. `Load Function reference` leaves File for Help as `Function Reference`. Notices move to the right of the bar.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Tags:** release/v1

- [x] View holds Zoom In, Zoom Out, Reset Zoom, a separator, then Diagnostics. Each zoom item triggers the same action as its chord and shows that chord as shortcut text.
- [x] Help holds Function Reference, which does what `Load Function reference` does today.
- [x] File holds only what is built so far (New once `file-new/02` lands, Quit on native).
- [x] The persistence notice and Theme notices sit right-aligned in the top bar.
- [x] The top-bar test asserts the new menu titles and the absence of any Function reference item in File.
- [x] Scoped gates for `console` pass, and the egui skill's guidance is followed.

## Comments

- The Settings menu stays after Help until `04` removes it; the top-bar tests list it last.
- File holds New (from `file-new/02`), then a separator and Quit on native; the web has New alone.
- The zoom items set `SourceView::requested_zoom`, which `show_source_scene` takes before it reads
  a chord, so a menu item and its chord go through the one `stepped_zoom` step.
- The top-bar test (`console::tests::the_bottom_panel_shows_tick_zero_and_run_clock_before_the_first_run`)
  asserts the menu titles in order from painted text, which cannot see inside a closed menu; the
  File contents are asserted by `kittest_tests::the_file_menu_offers_new_and_quit_and_no_function_reference`.
- New and Quit carry no shortcut text. ⌘N is not bound to a chord yet, so showing it would
  advertise a key that does nothing; the File items take their shortcut text when their chords land
  (`05`–`08`). Help → Function Reference replaces the Source without asking; making it the guarded
  Open is `06`'s, with the confirmation it generalises.
- The zoom chords and the View menu's shortcut text come from one table (`ZoomCommand::key`), so
  the chord shown is the chord that works. The persistence notice is truncated to the space the
  menus leave, with its whole text on hover
  (`kittest_tests::a_long_notice_in_a_narrow_window_stays_right_of_the_menus`).
