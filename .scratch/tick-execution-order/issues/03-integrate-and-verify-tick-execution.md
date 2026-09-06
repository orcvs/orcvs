# 03 — Integrate playback and verify the execution contract

Status: resolved
Tags: release/v1
Blocked by: 02

**What to build:** Complete integration and verify [ADR 0032](../../../docs/adr/0032-schedule-tick-execution-by-dependency.md) against real Source, Playback, and supported targets.

- [x] Replace activation fixtures that manually type `**` with real Bang-producing Functions; retain explicit manual-no-op regression coverage.
- [x] Preserve tests for current note delivery, Timed Play expiry, zero velocity and length, cancellation, replacement ownership, output failure, and recovery. A silent fixture must not make these pass vacuously.
- [x] Verify repeated producer output activates on every intended Tick, stopping the producer prevents replay, and independent ready MIDI roots have deterministic command order.
- [x] Exercise invalid embedded Bangs and incomplete inputs through Source execution and the Language Map, preserving the distinction between syntax diagnostics and Tick diagnostics.
- [x] Review the full implementation diff and reconcile runtime docs with ADR 0032. Mark `language-map/06` and `spatial-tick-planning/02` complete only when their actual acceptance criteria pass.
- [x] Capture the accepted self-contained prototype on a throwaway branch, separate from production changes, and record its branch/commit on the originating issue.
- [x] Run both affected crates' scoped gates, `mise run check`, persistence verification, WASM build/browser gates, and rustdoc warnings. Record exact failures and distinguish existing tooling failures from regressions.
- [x] Review parser/public-interface, ordering/concurrency, native/WASM, persistence, and performance risks. Do not make performance claims without a reproducible benchmark.

## Comments

2026-09-06: Draft test-only edits exist in `orcvs/src/playback.rs` and `orcvs/src/midi.rs`. Fixtures use `.=0101` two rows above a MIDI root and `.=0102` to stop pulses. They have not been compiled or tested after editing. Some repeated writes of unchanged comparison operands remain and should be removed where they obscure continuous producer behavior. Runtime scheduling is not implemented yet.

Earlier fallback verification passed 232 `orcvs` tests and scoped checks, but that evidence predates these drafts and does not validate ADR 0032. `mise run check` previously stopped at the existing tooling-contract check expecting an `actions/checkout` v4 annotation. Re-run the gates; do not treat the previous failure as an automatic exemption.

2026-09-06: Resolved. Native nextest passes 387 tests; the persistence gate passes 442 tests plus checks, doctests, and rustdoc; WASM default/persistence builds pass; and five headless Firefox regressions pass. `mise run check` was rerun and still stops at the pre-existing tooling-contract requirement for a 40-character `actions/checkout` v4 pin. The accepted prototype is isolated on `prototype-operation-ordering` at `841f696`. `language-map/06` is resolved. `spatial-tick-planning/02` remains open because horizontally adjacent authoring and the broader Directional/Self-Banging activation surface are outside ADR 0032's initial fixed-Portal scope.
