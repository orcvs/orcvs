# 07 — Open a Source File

**What to build:** File > Open… (⌘O) on native picks a `.orcvs` file with a native dialog and opens it as the environment, through the same Open `file-new/01` names.

**Blocked by:** 05 — Read and write a Source File; 06 — Track the open Source File.

**Status:** resolved

**Tags:** release/v1

- [x] `rfd` is added through `rust-dependency-change`, synchronous API, native targets only, with its rationale recorded; `mise run audit_deps` passes.
- [x] The dialog filters to `.orcvs` and still allows any file.
- [x] Unsaved changes are guarded as `06` specifies before the dialog opens.
- [x] A refused file leaves the running Source, path and unsaved state untouched and shows the refusal as a notice.
- [x] A read file becomes the environment; its path becomes the open file and the Source counts as saved.
- [x] Open does not appear on web, and ⌘O does nothing there.
- [x] Tests cover open, refusal and a cancelled dialog without driving a real dialog.
- [x] Scoped gates for `console` pass, and `mise run check_wasm` still compiles.

## Comments

**`rfd` 0.17.2**, native targets only, `default-features = false, features = ["xdg-portal"]`;
the rationale is on the entry in `console/Cargo.toml`. `wayland` is dropped because it only mints
a parent-window identifier and the console passes no parent. `xdg-portal` is the Linux backend
with no build-time system library (it `dlopen`s `libdbus-1`, falling back to `zenity`); `gtk3`
would need GTK headers in CI. `Cargo.lock` gains only `rfd` and `pollster`; the macOS `objc2`
0.6 family and `windows-sys` 0.61 were already locked. `mise run audit_deps` passes (the duplicate
warnings it prints were there before).

**Shape.** `FileCommand::Open` (menu item `Open…`, ⌘O) runs through `run_file_command`, which asks
with `06`'s predicate before anything opens. Confirmed or unasked, `Console::choose_and_open` is
the shipped entry: the synchronous `rfd::FileDialog::pick_file`, then `open_picked(Option<PathBuf>)`,
which opens nothing for a cancelled dialog and otherwise calls `open_path`. `open_path` reads
through `source_file::read_source_file` (at most one byte past the longest possible Source File,
256 × (256 + CRLF), so a large file picked by mistake is not read whole; a text that long is
already refused where the whole file would be) and `orcvs::source::file::read`, then the same
`Console::open` New uses, then names the file. Tests call `open_picked` and `open_path` with the
paths a dialog would answer; a menu Open is driven only as far as the question before the dialog.

**Notices.** A file that cannot be read, or that `file::read` refuses, is a notice in the top
bar's Notices menu (with its Dismiss), beside the Theme and settings notices, and is reported to
the developer console. That menu is the existing channel that holds a problem until the viewer
dismisses it; the persistence label is a single start-up notice with its own key. The Source File
notices are the console's own list (`Console::file_notices`), not the Theme registry's.

**Filters.** The dialog offers "Orcvs Source File" (`orcvs`) and "All files" (`*`). Windows and
the portal show them as a choice. On macOS `rfd` 0.17.2 merges every filter into one
`setAllowedFileTypes` list (`backend/macos/file_dialog/panel_ffi.rs`, `add_filters`), where `*` is
a literal extension rather than a wildcard, so the pair would admit `.orcvs` alone. macOS
therefore adds no filter and the panel allows any file. Neither half is reachable by a test.

**Tests** (`console::kittest_tests`): `an_opened_source_file_is_the_environment_and_saved`,
`a_refused_file_opens_nothing_and_says_why` (refused and unreadable), `a_cancelled_open_changes_nothing`,
`open_asks_before_discarding_unsaved_changes` (menu and ⌘O); the File menu test lists Open… with
its chord. `source_file::tests` cover the read, the refusal text and the length bound.
