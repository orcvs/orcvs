# 01 — Restore Source through the shipped storage path

**What to build:** Wire `Source` into eframe storage, so the application saves the current Source
revision and restores it on the next start. The model layer already round-trips; nothing calls it.

**Blocked by:** None.

**Status:** resolved

**Tags:** release/v1

- [x] `eframe::App::save` stores the current `Source` revision. It replaces the commented-out
      implementation at `shell/src/console.rs:403`, which stored `self` — the whole `Console` —
      and so contradicted `source-playback-engine/18`'s decision that `Source` is the one
      supported persistence root.
- [x] `Console::new` reads `cc.storage` and restores that revision. It reads nothing today
      (`shell/src/console.rs:153`).
- [x] Restored Source rebuilds every derived view. The Language Map, Glyphs, diagnostics, and
      parsed Expressions are derived, never stored.
- [x] Absent stored state starts the ordinary default Grid. A malformed stored value is refused,
      is reported, and starts the default Grid rather than a partly restored one.
- [x] The `persistence` feature gates the path. `shell/Cargo.toml:24` declares it over
      `eframe/persistence` and `orcvs/persistence`. A build without the feature keeps today's
      behaviour and compiles.
- [x] Native and WASM store the same shape through the same code. Only the eframe storage backend
      differs.
- [x] Automated tests cover the save call, the restore call, an absent value, and a malformed
      value. They run under `mise run test_persistence`.
- [x] `source-playback-engine/18`'s Answer records that `App` and `Console` stay excluded and that
      this ticket adds the storage call for the `Source` root, so the two statements do not read
      as a contradiction.

## Comments

2026-09-09: Raised by the release-candidate audit. `v1-release/02` holds four items that prove a
native and WASM save–restart–reload path, and
`v1-release/definition-of-done.md` requires the same. No such path exists, and before this ticket
no issue owned it, so those items had nothing to test.

2026-09-09: Resolved. `Console::new` reads `cc.storage` through
`shell/src/persistence.rs`, the console's storage seam: it starts the stored `Source` revision,
the ordinary default Grid when storage holds none, and — for a value that is present but does not
decode — the default Grid after reporting the refusal, so a partly restored Source never reaches
the console. `eframe::App::save` stores the current revision under `eframe::APP_KEY` through the
same module. `Orcvs::with_source`, `Orcvs::with_source_and_output_adapter`,
`SourceCommander::with_source` and the persistence-gated `SourceCommander::read_source` and
`Orcvs::source` are the seam the shell reaches the root through; every existing constructor now
runs through the same path. Native and WASM share the code and differ only in eframe's storage
backend.
