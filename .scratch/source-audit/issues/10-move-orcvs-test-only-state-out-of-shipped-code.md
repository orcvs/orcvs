# 10 — Move orcvs test-only state out of shipped code

**What to build:** `orcvs` production Ticks do no work that only tests read, and its public API carries no test-only hooks. Today the Tick computation state records turns and interpretations on every Tick behind three dead-code allowances (`source/tick/execution.rs`); `InMemoryOutputAdapter`, with its fail-next-submission hook, is public shipped API; the Playback path handles an error `MidiOutputAdapter::install_connection` can never return; and a few Playback tests assert on wall-clock `std::thread::sleep`.

The in-memory adapter is used outside the crate — `console/tests/wasm.rs`, `console/benches/paint.rs` and a `compile_fail` doctest in `app.rs` — so crate-private test support cannot serve it. The precedent is `test-grid-shapes`: a feature only dev-dependencies enable.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Fields and accessors only tests read are compiled only under test, or derived by tests below the shipped entry point.
- [x] The in-memory output adapter is compiled only under test or behind a feature only dev-dependencies enable, as `test-grid-shapes` is, and the console's tests, benches and the doctest still reach what they need.
- [x] Connection install is infallible in its signature, or a real failure is produced and tested.
- [x] No Playback test sleeps to manufacture elapsed time. The tests that sleep measure the Run Clock, which reads `web_time::Instant::now()`, so paused Tokio time alone does not move it: the Run Clock reads a clock tests can control, or the tests are restructured.
- [x] Clippy passes without the dead-code allowances this ticket removes.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Every claim still holds. Criteria 2 and 4 corrected: the adapter has users outside the crate, and the sleeping tests measure a clock Tokio's paused time does not drive.

**2026-09-25 — implementation.** Implemented in orcvs/orcvs#147 (epic PR 5): test-only Tick records under `cfg(test)`, `InMemoryOutputAdapter` behind dev-only `test-output-adapter`, infallible `install_connection`, no wall-clock sleeps in Playback tests. Resolve on merge.

**2026-09-25 — resolved.** Merged in orcvs/orcvs#147 (`43e5a44f`); every criterion verified on `main`.

**2026-09-25 — criterion 4 narrowed.** The criterion targets sleeps that manufacture elapsed time for the Run Clock, which #147 removed. The harness poll `wait_until` in `orcvs/src/playback.rs` still sleeps 1 ms between checks while it waits, with a wall-clock deadline, for a condition another OS thread's blocking adapter sets; it predates the audit and #148 kept it. It is out of this ticket's scope; replacing it with a notification is separate work if wanted.
