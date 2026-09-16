# 01 — Deepen MIDI selection within Playback

**What to build:** Keep console discovery and MIDI destination selection working through a
restricted, nonblocking selection handle, while concentrating destination publication and the
selection protocol in the MIDI selection module. The Playback Engine's task continues to own
all connection and Playback mutations.

**Blocked by:** None — can start immediately.

**Status:** resolved

Superseded in part by `midi-port-ownership`: discovery and port opening moved to the console;
Playback now receives `Install { destination_id, connection }` on the ordered queue. The ownership
guarantees below still hold.

## Agreed design

- MIDI selection owns destination publication. The shared output adapter interface handles
  command delivery and safety reset; generic Playback handles carry no MIDI destination state.
- Selection crosses the seam as explicit queue data processed inside the engine's task; callers
  cannot supply arbitrary closures that mutate the Playback Engine's entire state.
- MIDI-specific construction establishes the destination subscription before the adapter
  moves into the Playback task and connects the selection module to that task.
- Console reads remain nonblocking. Selection handles retain weak ownership and cannot keep
  the Playback task alive or acquire Playback lifecycle control.

## Acceptance criteria

- [x] A running Orcvs supplies a restricted selection handle that discovers destinations,
      shows the selected identity, and selects the same connection subsequent Playback uses.
- [x] MIDI destination publication is absent from the shared output adapter interface and
      generic Playback handles; a non-MIDI adapter constructs no unused destination channel.
- [x] Selection uses explicit queue requests processed by the existing Playback task; the
      arbitrary transition-closure mechanism is removed, including its test callers. (Was
      Discover/Select before `midi-port-ownership`; now Install with an open connection.)
- [x] MIDI-specific construction establishes publication before transferring the adapter to
      the task, and all production construction paths use that wiring.
- [x] Discovery failures and connection failures retain their existing reporting behavior.
      Successful queueing remains distinct from successful device selection.
- [x] Changing destination preserves safety reset, scheduled-note cleanup, and command order.
      Failed selection and reselection preserve the existing cleanup and diagnostic guarantees.
- [x] A selection handle cannot keep Playback alive, and requests or observations after owner
      loss report unavailability. The handle exposes no Playback lifecycle control.
- [x] Native and WASM construction preserve nonblocking console reads and existing behavior;
      each declared feature combination remains supported.
- [x] Tests cross the production selection and Playback interfaces with the existing fake
      MIDI backend, covering discovery, selection, failures, output delivery, safety reset,
      scheduled-note cleanup, and handles outliving Playback.
- [x] ADR 0041 records the agreed publication placement and explicit selection requests,
      resolving its open ownership question while preserving single-task mutation ownership.

## Verification

Run the repository's scoped formatting, clippy, and nextest gates for orcvs and console with
`PROPTEST_CASES=32`. Run the relevant doctests and rustdoc checks for changed Rust interfaces;
run the WASM compile gate if platform construction changes. Preserve the existing capability
restriction tests. Leave browser execution, the complete target matrix, benchmarks, and full
property-test case counts to CI as the repository contract requires.

## Context

Implements the MIDI selection design confirmed during the architecture grilling conversation.
Related decision ticket: playback-actor/15, “Decide whether the actor keeps a general transition
hatch.” That ticket is context, not a blocker: publication ownership, explicit requests, and
MIDI-specific construction were settled together in the conversation. Do not close or modify
that parent ticket as part of this work; record the adopted decision in ADR 0041.

Keep this as one complete slice: console discovery and selection, task-owned transitions,
publication wiring, removal of the old mechanisms, and behavioral verification land together.
No second Playback task, connection owner, or output adapter is introduced.

## Completion evidence

Changed: MIDI selection owns its publication value and restricted handle. MIDI construction
subscribes before handing the adapter to the Playback task. Explicit requests share the one
lifecycle queue; queued transition closures and generic destination channels are removed.
Console and benchmark type annotations follow the capability-based construction. ADR 0041
records the decision; playback-actor/15 is unchanged.

Tests added or updated: production MIDI construction, selection-before-start ordering without
an intervening yield, immediate observation failure after dropping the owner (including a cloned
selection handle), output-only compile-fail capability example, and typed test-only clock probes.
Existing discovery, connection-failure, output-delivery, safety-reset, note-schedule, reselection,
and shutdown tests continue to cross the production selection and Playback seams.

Commands run (from the dedicated `midi-selection-ownership` worktree):

- `cargo fmt --all -- --check` — passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `PROPTEST_CASES=32 cargo nextest run --workspace --locked` — passed, 888 tests before review fix.
- `PROPTEST_CASES=32 cargo nextest run --workspace --tests --no-default-features --locked` — passed, 872 tests before review fix.
- `cargo clippy --package orcvs --package console --all-targets --locked -- -D warnings` — passed after review fix.
- `PROPTEST_CASES=32 cargo nextest run --package orcvs --package console --locked` — passed after review fix, 609 tests.
- `PROPTEST_CASES=32 cargo nextest run --package orcvs --package console --tests --no-default-features --locked` — passed after review fix, 593 tests.
- `PROPTEST_CASES=32 cargo nextest run --package orcvs --no-default-features --locked` — passed after review fix, 492 tests; unlike the workspace run, this also disables native MIDI.
- `PROPTEST_CASES=32 cargo nextest run --package orcvs --test midi_selection_handle --locked` — expected red for missing constructor, then green; review regression red with 2 passed/1 failed, then green with 3 passed.
- `cargo test --workspace --doc --locked` — passed, 15 doctests including seven compile-fail capability checks.
- `RUSTC_WRAPPER= RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked` — passed.
- `mise run check_wasm` — passed after review fix: all-target clippy plus default and feature-off application builds.
- `node --test scripts/tests/roadmap.test.ts` — passed, 10 tests.
- `node scripts/roadmap.ts > /dev/null` — passed.
- `git diff --check` and `git diff --cached --check` — passed.

Initial focused tests and rustdoc attempts hit sccache permission errors. The tests passed
after escalation; rustdoc still failed with the wrapper after escalation and passed with
`RUSTC_WRAPPER=`. An intermediate clippy run caught old benchmark type annotations;
those annotations were updated and the final gates passed.

Review — Standards: no findings. Spec: R1 (minor, original P2) confirmed and fixed.
R1 reproduced cached successful observations immediately after owner drop; the regression
failed before adding the weak-owner check and passed afterwards. Targeted re-review confirmed
R1 resolved, with no additional findings.

Not run: `mise run check`, `mise run check_merge`, `mise run test_wasm`, `mise run bench`,
the full target matrix, and 256-case property runs — deferred to CI.

Risks: internal Rust public construction/type signatures and concurrency wiring changed.
No language semantics, unsafe code, dependency declarations, feature declarations, or
performance claims changed. Existing feature-on/off behavior and WASM construction verified.
