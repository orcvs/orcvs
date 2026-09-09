# 01 — Restore Source through the shipped storage path

**What to build:** Wire `Source` into eframe storage, so the application saves the current Source
revision and restores it on the next start. The model layer already round-trips; nothing calls it.

**Blocked by:** None.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] `eframe::App::save` stores the current `Source` revision. It replaces the commented-out
      implementation at `shell/src/console.rs:403`, which stored `self` — the whole `Console` —
      and so contradicted `source-playback-engine/18`'s decision that `Source` is the one
      supported persistence root.
- [ ] `Console::new` reads `cc.storage` and restores that revision. It reads nothing today
      (`shell/src/console.rs:153`).
- [ ] Restored Source rebuilds every derived view. The Language Map, Glyphs, diagnostics, and
      parsed Expressions are derived, never stored.
- [ ] Absent stored state starts the ordinary default Grid. A malformed stored value is refused,
      is reported, and starts the default Grid rather than a partly restored one.
- [ ] The `persistence` feature gates the path. `shell/Cargo.toml:24` declares it over
      `eframe/persistence` and `orcvs/persistence`. A build without the feature keeps today's
      behaviour and compiles.
- [ ] Native and WASM store the same shape through the same code. Only the eframe storage backend
      differs.
- [ ] Automated tests cover the save call, the restore call, an absent value, and a malformed
      value. They run under `mise run test_persistence`.
- [ ] `source-playback-engine/18`'s Answer records that `App` and `Console` stay excluded and that
      this ticket adds the storage call for the `Source` root, so the two statements do not read
      as a contradiction.

## Comments

2026-09-09: Raised by the release-candidate audit. `v1-release/02` holds four items that prove a
native and WASM save–restart–reload path, and
`v1-release/definition-of-done.md` requires the same. No such path exists, and before this ticket
no issue owned it, so those items had nothing to test.
