# 01 — Restore Source through the shipped storage path

**What to build:** Wire `Source` into eframe storage, so the application saves the current Source
revision and restores it on the next start. The model layer already round-trips; nothing calls it.

**Blocked by:** None.

**Status:** resolved

**Claimed by:** `11-persist-source` / PR #38 — 2026-09-09. Recorded because this ticket was
implemented twice: the claim was invisible once the first branch's worktree was removed, and the
tracker has no field or status that says a ticket is being worked. See the last comment.

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
- [x] The `persistence` feature gates the path, and ships on. `shell/Cargo.toml:36-37` declares
      `default = ["persistence"]` over `eframe/persistence` and `orcvs/persistence`, so the binary
      anyone launches saves and restores. A build without the feature keeps today's behaviour and
      compiles; `--no-default-features` is the arm that proves it, in `check_pull_request` and in
      `check_wasm`'s second `trunk build`.
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
the console. `eframe::App::save` stores the current revision under an `orcvs_source` key through
the same module. `Orcvs::with_source`, `Orcvs::with_source_and_output_adapter`,
`SourceCommander::with_source` and the persistence-gated `SourceCommander::read_source` and
`Orcvs::source` are the seam the shell reaches the root through; every existing constructor now
runs through the same path. Native and WASM share the code and differ only in eframe's storage
backend.

2026-09-09: The shipped-application question is settled by default-on. `persistence` was an opt-in
feature, so every check above held for a build that no ordinary invocation produced: `mise run run`,
`cargo run`, the `.vscode` cargo tasks, and a plain `trunk build` all take the default set, and none
of them enabled it. `shell/Cargo.toml` now declares `default = ["persistence"]` — the cost is
`eframe/persistence` and `serde` in `orcvs`, both already in the locked graph, and the storage read
at start plus the write eframe already schedules.

The feature stays a feature, because the last item above requires the path to still compile out.
What proves it moved: the verification tiers used to pair a default run against a
`--features persistence` run, and with the default on those two are one configuration. The
pull-request tier now pairs the default run against `--no-default-features` for clippy, nextest and
doctests, and `check_wasm` pairs its two `trunk build` invocations the same way.
`scripts/check-tooling-contract.sh` pins both halves of every pair and pins
`default = ["persistence"]` beside them, because flipping the default back would collapse each pair
into two feature-off runs and leave the storage path compiled nowhere.
`mise run test_wasm` drops its flag and runs the browser's own configuration; `mise run
test_persistence` keeps its explicit `--features persistence`, which `orcvs` — still `default = []`
— needs for its own `--package orcvs --lib` line. `docs/tooling.md` records the arrangement.

2026-09-09: Review pass, and the merge of a duplicate implementation. A second branch
(`01-restore-source-through-the-shipped-storage-path`) implemented this same ticket independently
— see the `Claimed by` note below for why that was possible. Three reviews of it found two defects
that this branch shared, both in the "is refused, is reported" bullet. Both are fixed here, and the
duplicate branch is dropped.

- A refused value was destroyed thirty seconds later. eframe's default `auto_save_interval` is
  thirty seconds and `store_source` wrote its key unconditionally, so a value refused for a
  recoverable reason — a format skew, a truncated write, one bad byte — was overwritten with the
  empty default Grid before a viewer could act. `StoredSource::Refused` now carries the value,
  `Start` hands it to the console, and the first `save` moves it to `orcvs_source_refused` before
  writing. The save still happens: a console that stopped saving after a refusal would lose the
  session that followed instead.
- "Reported" reached no viewer. `crate::report` reaches a terminal or a browser developer console,
  which is the right channel for a developer and no channel at all for the person looking at a Grid
  that is not theirs. A refusal now also raises a dismissible notice in the menu bar in the error
  colour, beside the existing MIDI status line. The Diagnostics window was the other candidate and
  is the wrong one: it opens on request and reports the running frame, where this is a start-up
  answer about the Source in front of the viewer.

Also from the duplicate: the storage key is `orcvs_source`, not `eframe::APP_KEY`. That key names
the whole App value, and `source-playback-engine/18` settled that the Console is a runtime
coordinator with nothing to restore; storing a Source under "app" reads as storing the Console
again. `shell/tests/wasm.rs` and the storage tests follow the key.

The duplicate's own reporting fix — enabling `tracing/log` for the whole `shell` graph — was
deliberately **not** taken. Cargo unifies features, so it bridges every `tracing` event in the
build to `log`, including `Source::set`'s per-write `debug!`, and `main` wires `WebLogger` at
`Debug`. `shell/src/report.rs` pays the second channel only where a call site asks for it, which is
the same fix without the traffic.

Deliberately not changed: a save taken while the transport runs stores tick-written Cells, because
`execute` commits `TickPlan.writes` into the Source. Bangs are Grid Cells; that is Orca's model,
not runtime state leaking into the root.
