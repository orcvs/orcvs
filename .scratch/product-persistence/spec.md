# Product persistence

## Goal

A Source revision survives a restart on native and a reload in the browser, through the
storage path the shipped application actually uses.

## Where this stands

`source-playback-engine/18` repaired the `persistence` feature and decided its shape. Persistence
has one supported root, `Source`, whose canonical state is `Grid` plus inner Cells.
Deserialization validates the dimensions, the Cell count, and the Cell bytes, then rebuilds every
derived view. `App` and `Console` were excluded on purpose, because they are runtime coordinators.

That work stopped at the model. The application never calls it:

- `eframe::App::save` is commented out at `shell/src/console.rs:403`.
- `Console::new` never reads `cc.storage` (`shell/src/console.rs:153`).
- No `set_value`, `get_value`, or `APP_KEY` call exists anywhere in `shell/src`.

So the feature compiles, the model round-trips under test, and a user still loses the Source when
the application stops.

## What this effort adds

The missing wiring, and only that. The stored shape stays the one `source-playback-engine/18`
decided: the `Source` root, not the `Console`. The commented-out line stored `self`, the whole
`Console`, which is the shape that ticket rejected.

## Out of scope

- A file format, a document model, or more than one stored Source.
- Cross-version compatibility. `v1-release/definition-of-done.md` defers it while Orcvs is
  pre-release.
- Any change to what `Source` serializes.
