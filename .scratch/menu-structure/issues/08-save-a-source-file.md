# 08 — Save a Source File

**What to build:** File > Save (⌘S) writes the Source to the open file; File > Save As… (⇧⌘S), or Save with no open file, picks a path with a native dialog first.

**Blocked by:** 05 — Read and write a Source File; 06 — Track the open Source File; 07 — Open a Source File.

**Status:** resolved

**Tags:** release/v1

- [x] Save writes Source File text to the open path and clears the unsaved marker.
- [x] Save As defaults to the `.orcvs` extension, and its path becomes the open file.
- [x] A failed write leaves the unsaved marker set and shows the error as a notice.
- [x] The write does not leave a truncated file on failure: write beside, then rename.
- [x] Neither item appears on web, and the chords do nothing there.
- [x] Tests write under an isolated directory and read the result back through `05`.
- [x] Scoped gates for `console` pass, and `mise run check_wasm` still compiles.

## Comments

**Shape.** `FileCommand::Save` (⌘S) and `FileCommand::SaveAs` (⇧⌘S) sit between Open… and the
separator before Quit, native only, through the same `run_file_command` dispatch; saving discards
nothing, so neither asks. `Console::save_file` writes to the open file's path, or runs Save As
when there is none. `Console::save_file_as` is the shipped entry: the synchronous
`rfd::FileDialog::save_file`, filtered to Source Files, opened in the open file's folder with its
name or `Untitled.orcvs`, then `save_picked(Option<PathBuf>)`, which saves nothing for a cancelled
dialog. `save_picked` gives a bare name the `.orcvs` extension (a portal dialog can answer one;
another extension is the viewer's choice and kept) and calls `save_to`.

**Write beside, then rename.** `source_file::write_beside_then_rename` writes to a hidden
`.name.pid-n.saving` file created new in the target's own folder (a rename across file systems is
a copy), `sync_all`s it, and renames it over the target; any failure removes the file beside and
leaves the target as it was. The text and the Cells it was written from are read under one lock,
so a Tick between them cannot leave the file recorded as saved against Cells it does not hold.
Consequences not handled: the replaced file's permissions are not carried over, and a target that
is a symlink is replaced by a regular file rather than written through.

**Errors.** A failed write keeps the open file and the unsaved marker and raises a notice in the
Notices menu, the channel `07` uses.

**Tests.** `console::kittest_tests`: `save_writes_the_open_file_and_clears_the_marker` (menu and
⌘S, read back through `orcvs::source::file::read`), `save_as_writes_the_picked_path_and_opens_it`
(bare name gets `.orcvs`), `a_cancelled_save_as_writes_nothing`,
`a_failed_save_keeps_the_marker_and_says_why` (a directory at the target: nothing beside it, the
open file untouched), and `each_file_chord_is_its_command` (Shift makes ⌘S Save As; Alt and
repeats are no chord). `source_file::tests`: the write replaces a file whole and leaves nothing
beside it, a failed write leaves its target, and the extension rule. Every test writes under its
own `TempDir`. Save As itself is not driven from the menu or ⇧⌘S, since that opens the dialog.

**From review (fixed).** Save As appended `.orcvs` after the dialog had already asked about replacing the name it was given, so `song` could replace an existing `song.orcvs` without a prompt; `save_picked` now refuses with a notice when the name the extension makes already exists. The extension is appended unless the name already ends `.orcvs` (any case), so a dotted name like `loop-v1.2` becomes `loop-v1.2.orcvs` rather than being saved without it. An Open whose file reads but whose Source cannot start now raises a notice as well (`07`'s path). `discard_asking_first` asks `asks_before_discarding` itself, so no discarding action can forget to.

**From review (not changed; open).**
- The dialogs are built without `set_parent`, so on Windows and Linux they are not owned by the console window and can open behind it while the frame waits. The discard confirmation's `fn(&mut Console)` has no `eframe::Frame` to take a window handle from; threading one through is a follow-up.
- Writing beside then renaming replaces a symlinked Source File with a regular file, resets its mode, and cannot save into a directory where files can be edited but not created. Recorded above as consequences; writing in place would give up the no-truncation guarantee.
- The unsaved comparison copies the 64 KiB Source once per revision under the read lock (once per Tick while playing). Judged cheap against the per-frame Render Frame derivation, which already copies it every frame; a borrowed comparison would need a new `orcvs` accessor.
- `serde-saphyr`, `serde_json` and `toml` are still built for the web, where after `09` no Theme document is decoded; `toml` still serves the native config. Moving them to the native table is its own dependency change.
