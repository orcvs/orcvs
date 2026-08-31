# 01 — Narrow shell access to MIDI device selection

**What to build:** Let the console discover and select a MIDI destination through a restricted
handle that configures the running Orcvs's actual output connection without granting the console
authority to start, stop, disconnect, or observe Playback.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] A MIDI-backed running Orcvs supplies a handle limited to destination discovery, selection, and selected-identity inspection.
- [x] Selecting a destination through the handle configures the same output adapter and connection used by subsequent Playback.
- [x] The shell-owned MIDI configuration keeps the destination list and visible status while storing no complete Playback Engine handle.
- [x] The running Orcvs exposes no general accessor that clones its complete Playback Engine.
- [x] Playback start, stop, disconnect, observation, and diagnostic ownership remain with the running Orcvs.
- [x] A compile-fail API test proves the MIDI selection handle cannot control or observe Playback lifecycle.
- [x] Existing discovery, selection, error-status, Playback, native, persistence, and WASM behavior remains green.

## Comments

This is capability narrowing, not a user-visible behavior change. Keep the current shared Playback
Engine state underneath the handle; do not introduce a second engine, output adapter, connection,
clock, or cancellation owner.

The primary behavioral seam is end to end: select a destination through the restricted handle,
drive Playback through the running Orcvs, and observe the selected fake connection receiving the
output. Do not test private fields or synchronization implementation details.

Implemented with a non-owning handle backed by a weak reference to the running Orcvs's Playback
state. TDD proves that selection configures the connection used by Playback, and that retaining the
handle cannot delay synchronous shutdown or operate after the running Orcvs is dropped. Five
compile-fail doctests enforce removal of the complete-engine accessor and the absence of start,
stop, disconnect, and observation authority.

Two-axis review found and fixed two defects before commit: the first strong handle could outlive
Orcvs and delay all-notes-off, and the initial API was available on unsupported platforms. The
handle now reports owner loss explicitly and is limited to the supported desktop targets. The full
repository gate, persistence coverage, dependency policy, WASM build, and Firefox WASM tests pass.
