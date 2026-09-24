# 06 — Track the open Source File

**What to build:** On native the console knows the path of the Source File it has open, if any, and whether the Source changed since that file was opened or saved. The window title says both. Every action that discards the Source asks first when there are unsaved changes.

**Blocked by:** file-new/03.

**Status:** resolved

**Tags:** release/v1

- [x] The title reads the file name, or `Untitled`, with an unsaved marker while the Source differs from what was last opened or saved.
- [x] New, Help > Function Reference, File > Quit and closing the window ask before discarding unsaved changes, through the confirmation `file-new/03` introduced. Cancelling changes nothing.
- [x] An unchanged Source asks nothing.
- [x] Autosave still stores the Source, and a restart restores it; whether the restored Source counts as unsaved against its file is decided and recorded here.
- [x] Web builds track nothing and show no marker.
- [x] Tests drive each discarding action through the console with and without unsaved changes, confirmed and cancelled.
- [x] Scoped gates for `console` pass.

## Comments

**Unsaved is a comparison of Cells.** `console/src/source_file.rs`'s `OpenSourceFile` keeps the
Source's Cells as they stood when last opened or saved, and a Source is unsaved while its Cells
differ. A revision counter was rejected: the console has no Undo, so typing a character and
deleting it again leaves nothing to save, and a counter would still say unsaved. Playback's Ticks
write the Source, so a running program can make its file unsaved without a key being pressed —
that is what Save would write.

**Cheap per frame.** The comparison copies the Source, so it runs only when the Source is at a
revision it has not answered for. `orcvs` gains `Source::revision` / `SourceCommander::revision`,
a `RevisionId` minted by every write (edit, block write, Tick commit — including an empty one) and
never by a read, unique across Sources so an identity from a replaced Source never matches its
successor. The console reads it each frame under the lock without copying a Cell and reuses the
last answer while it is unchanged; while Playback runs, it compares once per Tick.
`SourceCommander::read_source` lost its `persistence` gate so the console can read the revision
and the Cells under one lock (and, in `08`, write a Source File).

**What each Open leaves.** Every Open (New, the Function Reference, and `07`'s file) opens a saved
Source: New's empty Source and the Function Reference are Untitled and unmarked, so New over an
unedited Function Reference asks nothing on native. Only the Open from a file names a file.

**Restored Source (decision).** Autosave still stores the Source and a restart restores it. Storage
keeps the Source, not the file it came from, so a restored Source is Untitled and is unsaved
against the empty Source a fresh console opens on: a restored Source holding content is marked,
and New, Quit, closing the window and the Function Reference ask before discarding it. It exists
nowhere but the autosave, so asking is the safe side. Persisting the path (so a restored Source
could be compared against its file) is left open.

**Title.** `• name — Orcvs` while unsaved, `name — Orcvs` otherwise, `Untitled` without a file;
sent with `ViewportCommand::Title` only when it changes (`Console::shown_title`).

**Guards.** New and Help → Function Reference go through `discard_asking_first` with the `06`
predicate on native. Quitting is guarded at the window's close request alone: File → Quit sends
`ViewportCommand::Close`, and `Console::guard_close` answers any close request with unsaved changes
with `ViewportCommand::CancelClose` and the Quit question, whether it came from File → Quit, the
close button or egui's own quit shortcut (Ctrl+Q on Linux and Windows). A confirmed Quit sets
`closing`, so the close it sends is let through. The web keeps `source_is_written()` for New and
its unasked Function Reference, tracks nothing and sends no title.

**Native app-menu Quit ends without asking.** winit's default macOS app menu (Hide, Hide Others,
Services, Quit ⌘Q) is kept. Its Quit, like the Dock's Quit, log-out and shutdown, sends
`NSApplication::terminate:`, which ends the process with no close request (winit 0.30 implements
no `applicationShouldTerminate:`), so it ends without the unsaved-changes question. File → Quit
therefore shows no ⌘Q shortcut text.

**Chords.** ⌘N, ⌘O, ⌘S and ⇧⌘S (Ctrl on Linux and Windows) run their File commands on native
through `FileCommand` and `Console::run_file_command`, the dispatch the menu items use. Like the
Zoom chords they are read only while the Source holds the keys (ADR 0048). The console binds no
Quit chord. The File items show their chord as shortcut text on native only; the web shows none
and binds nothing.

Tests (`console::kittest_tests`): `the_window_title_names_the_source_and_marks_unsaved_changes`,
`the_function_reference_asks_before_discarding_unsaved_changes`,
`quit_requests_a_close`,
`closing_the_window_asks_before_discarding_unsaved_changes`,
`closing_an_unchanged_window_is_let_through`,
`the_file_chords_run_their_commands_and_never_the_source`,
`the_file_chords_run_nothing_while_the_keys_are_elsewhere`,
`a_restored_source_is_untitled_and_unsaved` (persistence), and
`the_file_menu_offers_its_items_with_their_chords_and_no_function_reference`; the `file-new/03`
tests still cover New asked, cancelled and unasked. `source_file::tests` and
`orcvs` `every_write_and_only_a_write_mints_a_new_revision` cover the predicate.

**From review (open product question).** The File chords follow ADR 0048 and run nothing while a
control holds the keys, so ⌘S does nothing while the Bpm field is focused or a menu is open. A
terminate hook would let the native app-menu Quit ask as well.
