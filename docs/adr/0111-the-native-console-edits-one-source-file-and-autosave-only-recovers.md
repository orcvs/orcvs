# The native console edits one Source File, and autosave only recovers the session

Status: proposed on 2026-11-15. Drafted for Toby from the built console, to record what the undated
`.scratch/menu-structure/spec.md` ("A document model on native", "Web file I/O is deferred",
"Settings are config-file only") decided and no ADR states. It decides nothing the code does not
already do. Extends the per-key fallback [ADR 0079](0079-source-cells-are-configurable-rectangles.md)
gives the `[cell]` keys to every settings key, on native and, through
[ADR 0091](0091-the-web-console-reads-its-settings-from-its-page.md), on the web.

The console opens, saves and discards Sources, and reads a settings file, but no ADR says what the
open file is, when a discard asks first, or how a faulty settings file falls back.
`docs/requirements/console.md` lists all of it as built, awaiting a record
([requirements-snapshot/13](../../.scratch/requirements-snapshot/issues/13-ground-the-native-document-model.md)).
The spec decided it in prose; the code below is the behaviour this record keeps.

## Decision

**On native the console tracks one open Source File: its path, if any, and the Cells it held when
last opened or saved.** `OpenSourceFile` in `console/src/source_file.rs` holds both. The Source is
**unsaved** while its Cells differ from those saved Cells, so typing a character and deleting it
leaves nothing unsaved, and a Tick's write can make the Source unsaved. Opening a file names it and
is saved. New, an example, the Function reference and a restored autosave open Untitled; a restored
Source is unsaved unless it is empty, because it is compared with the empty Source
(`Console::new`).

**The window title names the open file, or `Untitled`, and marks it while unsaved:**
`• loop.orcvs — Orcvs`. `Console::show_title` sends it when it changes.

**Every native action that would discard unsaved changes asks first.** New, Open, Help → Function
Reference, each Help → Examples item, File → Quit and the window's close button each show one modal question with Discard and Cancel while the Source is unsaved, and act at once
when it is not (`Console::discard_asking_first`, `Console::guard_close`, the `*_CONFIRMATION`
constants in `console/src/console/files.rs`). Save and Save As discard nothing and never ask.

**Save writes the file; autosave is session recovery.** Save writes the open file, or runs Save As
when none is open. Save As offers the open file's name, or `Untitled.orcvs`, in its folder, and
appends the `.orcvs` extension unless the picked name already ends in it, in any case, so
`loop.txt` is saved as `loop.txt.orcvs` and `loop-v1.2` as `loop-v1.2.orcvs`
(`with_source_file_extension` in `console/src/source_file.rs`); it refuses, with a notice, to replace
an existing file at the appended name, because the dialog confirmed a different one. A write goes
beside the file and is renamed over it, so a failed save never truncates it and raises a notice.
Separately, with the `persistence` feature, eframe's periodic and closing save stores the running
Source under `orcvs_source_rwc` (`persistence::SOURCE_KEY`). Storage keeps the Source and never its
file, so a restart restores the Cells, Untitled. Neither path writes the other: Save does not touch
storage, and autosave never writes a file.

**The web has no Open, Save or Save As.** Its File menu holds New. It tracks no file, so it asks
before discarding whenever any Cell of the running Source is written (`Console::source_is_written`):
New and each example ask, with their own wording. Help → Function Reference opens without asking on
the web. A web console opened at `/play/` with no link autosaves as native does; a Linked Console
and the Small Console neither restore nor store (SITE-021, SITE-010).

**A faulty settings file costs only the keys it affects, and says why.** `console/src/config.rs`
reads `~/.orcvs/config.toml` on native, and the page's `orcvs_config` text on the web, with one
reader (`parse`):

- A missing file is the defaults with no notice.
- A file that cannot be read, is over 64 KiB, or is not UTF-8, and text that is not valid TOML, is
  the defaults with one notice naming the file.
- A known section that is not a table leaves its keys at their defaults, with a notice.
- An unknown top-level value that is not a table, and each unknown key in any table, is ignored
  with a notice naming it; the known keys still apply. An unknown table with no keys, such as a
  bare `[glitch]` header, is ignored without a notice.
- A value of the wrong type or out of its range falls back alone, with a notice naming the key; the
  other keys stand.
- A Theme identity no Theme answers to falls back as CONS-067 states.

The notices go to the settings notice channel that Theme file problems use (`ThemeRegistry::notice`).

## Considered options

- **Open on the web without Save.** The spec rejected it as half a feature: `rfd` picks a file on
  wasm32 but cannot save one there (0.17.2), and a download through `web-sys` or the Chromium-only
  File System Access API is its own work. The web keeps autosave and, since ADR 0091, its page's settings.
- **Discard the whole settings file on any fault.** No source records it as considered; it is named
  here only because the per-key rule is the choice ADR 0079 already made for `[cell]`.

## Consequences

- `docs/requirements/console.md` can state the model as rules once this is accepted, replacing its
  "Built, awaiting a record" item, and CONS-131 can drop its pointer to an open decision.
- Every close request is asked about, but on macOS the app menu's ⌘Q ends the process without one,
  as the File menu's comment in `console/src/console/menu_bar.rs` records, so it quits without
  asking; no test covers it. File → Quit shows no chord for
  that reason.
- A Source a Tick keeps writing is unsaved for as long as Playback runs, so quitting a playing
  Source that was just saved can ask.
- Tests: `console/src/console/kittest_tests.rs`
  `the_window_title_names_the_source_and_marks_unsaved_changes`,
  `open_asks_before_discarding_unsaved_changes`, `an_example_asks_before_discarding_unsaved_changes`,
  `the_function_reference_asks_before_discarding_unsaved_changes`,
  `closing_the_window_asks_before_discarding_unsaved_changes`,
  `a_restored_source_is_untitled_and_unsaved`; `console/src/source_file.rs`
  `a_source_is_unsaved_while_its_cells_differ_from_the_ones_opened`; `console/src/config.rs`
  `an_unknown_key_is_a_notice_and_leaves_the_known_keys_applied`,
  `an_out_of_range_or_mistyped_value_falls_back_alone`,
  `a_malformed_file_is_the_defaults_and_a_notice_naming_the_file_and_line`.

## Open questions

Where the code and `.scratch/menu-structure/spec.md` differ, this record states the code.

- The spec lists New, Open, Function Reference, Quit and closing the window as the actions that ask.
  The code also asks before each Help → Examples item.
- The spec says nothing of the web asking. The code asks on the web before New and each example
  when any Cell is written, and opens the Function reference there unasked.
- The spec has Quit ask before discarding. On macOS, ⌘Q from the app menu quits without asking,
  because it sends no close request for the console to cancel.
- The spec's settings list (`theme.*`, `cursor_effects.glitch_*`) and its "no config file" for the
  web are older than ADRs 0079 and 0091, which replaced them; the code follows those ADRs, and the
  glitch keys now read as unknown keys.
