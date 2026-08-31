# 02 — Replace the egui input types in the application state

**What to build:** Define Orcvs input types in the `orcvs` crate and use them in `event_handler`.
Translate the toolkit's events into them in `shell`. This removes `use egui::{Event, Key}`, the only
toolkit reference in `app.rs`.

**Blocked by:** 01 — Extract the `orcvs` crate.

**Status:** resolved

- [x] `orcvs` declares its own key and input event types.
- [x] `event_handler` accepts the Orcvs types and no longer names an egui type.
- [x] `shell` holds the translation from `egui::Event` and `egui::Key`, and nothing else translates.
- [x] The translation covers every event the current handler acts on, and drops the rest explicitly.
- [x] `app.rs` compiles with no toolkit dependency in scope.
- [x] Existing keyboard behaviour is unchanged.

## Comments

581 lines of application logic sit behind this one import. Removing it moves all of them into the
fast test path.

Model only the input this application uses. A general reimplementation of `egui::Key` is not the
goal, and a partial copy that pretends to be general is worse than a small explicit set.

Make the translation total and visible. An event the shell silently drops is a keystroke the user
will report as a bug later.

Resolved by `a022b2d`. The shell translation has focused coverage for every handled key, text,
released keys, and explicitly dropped toolkit events. The combined issues 02–03 review found no
spec defects; its MIDI-boundary standards finding was addressed by issue 04.
