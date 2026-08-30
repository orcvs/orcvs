# Split the console into a core crate and a shell

**Goal:** Move the Orcvs domain out of the crate that depends on egui and eframe, so the language, the Source, and the Playback Engine compile and test without the UI toolkit.

## Why

ADR 0022 records the decision and the measurement behind it. Of roughly 5,900 lines in `console`, roughly 5,100 hold no reference to the toolkit. Only `console.rs` (45 references), `main.rs` (9), and `style.rs` (2) are genuinely coupled. `app.rs` holds 581 lines behind one import: `use egui::{Event, Key}`.

The cost of the current arrangement is compile time on every test. A cold workspace check costs 48 seconds and a cold test build costs a further 59 seconds, almost all of it `eframe`, `winit`, and `glutin`. `lang` builds and tests from nothing in 9 seconds. Property tests over the Grid, the Language Map, and the Source will pay the full UI compile for logic that never draws anything.

The boundary already exists in the documents. ADR 0001 separates Source interpretation from the Playback Engine. CONTEXT.md separates the language terms from the console terms. Neither separation can be enforced inside one crate.

## Crates

- `lang` — unchanged. Parsing and interpretation.
- `orcvs` — source, grid, glyph, cursor, render frame, playback, midi, opts.
- `shell` — console, style, main, web startup, MIDI device selection, and the toolkit dependency.

`orcvs` stays one crate. The further cut along ADR 0001 waits for a measurement that asks for it.

## Vocabulary

CONTEXT.md defines Glyph, Marker, Cursor, and Render Frame as things "the console" draws or edits, and none of the four touches egui. The crate named `shell` therefore keeps those glossary sentences true without an edit. "Console" stays the domain word for the application the user sees.

`Orcvs` becomes the name of the running instance. That gives the word a second meaning, so it needs a CONTEXT.md entry that separates the system from the instance.

## Issues

- `issues/01-extract-the-orcvs-crate.md`
- `issues/02-replace-the-egui-input-types.md`
- `issues/03-name-the-running-instance-orcvs.md`
- `issues/04-move-midi-device-selection-into-the-shell.md`
- `issues/05-correct-the-stale-doc-comments.md`

## Order

This effort follows `ci-tiers` and precedes `property-testing`. A property written against `console` today would move crates immediately afterwards. The language migration in `orcvs-language-migration` rewrites `source/` heavily, so move the files before that rewrite, not after it.
