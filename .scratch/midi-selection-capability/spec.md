# Narrow shell access to MIDI device selection

## Problem Statement

The shell needs to discover MIDI destinations, choose one, and show the current choice, but it
currently receives a complete Playback Engine handle to do so. That handle also permits starting,
stopping, disconnecting, and observing Playback. The shell's MIDI configuration currently avoids
those operations by convention, but the crate boundary does not enforce that restraint. A future
shell change could therefore take ownership of Playback lifecycle behavior that belongs to the
running Orcvs.

## Solution

Give the shell a capability-limited MIDI selection handle backed by the same Playback Engine as the
running Orcvs. The handle exposes only destination discovery, destination selection, and inspection
of the selected destination. The running Orcvs remains the only application-level owner of starting,
stopping, disconnecting, and observing Playback.

The shell continues to own its discovered destination list and user-visible status. Selecting a
destination through the restricted handle configures the same MIDI output subsequently used by
Playback, without creating another engine, output adapter, connection, or lifecycle.

## User Stories

1. As a performer, I want the MIDI menu to list available destinations, so that I can choose where Orcvs sends musical output.
2. As a performer, I want selecting a MIDI destination to affect subsequent Playback, so that the menu and audible output cannot disagree.
3. As a performer, I want the selected destination to remain visibly selected, so that I can see the active output configuration.
4. As a performer, I want destination discovery failures to remain visible in the console, so that unavailable MIDI services are diagnosable.
5. As a performer, I want destination connection failures to remain visible in the console, so that a failed selection is not mistaken for success.
6. As a performer, I want Playback failures to remain visible in the console, so that narrowing the configuration API does not hide operational errors.
7. As a maintainer, I want the shell to receive only MIDI selection capabilities, so that presentation code cannot accidentally control Playback lifecycle.
8. As a maintainer, I want the running Orcvs to remain the owner of starting and stopping Playback, so that lifecycle behavior has one authoritative boundary.
9. As a maintainer, I want the running Orcvs to remain the owner of Playback observation, so that its cached Playback state cannot diverge from another caller's observations.
10. As a maintainer, I want device selection and Playback to share one engine and output adapter, so that selecting a destination configures the connection Playback actually uses.
11. As a maintainer, I want the restricted capability to exist only for MIDI-backed output, so that non-MIDI output adapters do not expose meaningless device-selection APIs.
12. As a maintainer, I want misuse of the selection handle to fail at compile time, so that the boundary is enforced rather than documented as a convention.
13. As a maintainer, I want existing native MIDI behavior to remain unchanged, so that the capability refactor does not alter the performer's workflow.
14. As a maintainer, I want WASM and non-desktop configurations to remain free of native MIDI selection code, so that platform support remains intact.
15. As a maintainer, I want no new synchronization primitive or connection owner, so that the existing Playback concurrency and shutdown guarantees remain authoritative.

## Implementation Decisions

- Introduce a public MIDI selection handle in the toolkit-free Orcvs crate.
- The handle shares the existing Playback Engine's internal state; it does not create or own a
  second engine, adapter, MIDI connection, clock, or cancellation lifecycle.
- Expose only three capabilities through the handle: discover destinations, select a destination,
  and inspect the selected destination identity.
- Make the handle available only from a running Orcvs whose output adapter is MIDI-backed.
- Remove the general method that returns a complete cloned Playback Engine from a running Orcvs.
- Keep Playback start, stop, disconnect, observation, diagnostics, and cached state behind the
  running Orcvs boundary.
- Keep the shell-owned MIDI configuration type responsible for the destination list and
  user-visible status.
- Replace the shell configuration type's complete Playback Engine field with the restricted MIDI
  selection handle.
- Continue routing Playback diagnostics from the running Orcvs to shell presentation separately;
  the selection handle does not gain observation authority.
- Preserve the existing shared-state locking and poisoned-lock recovery behavior.
- Preserve native and WASM platform guards. A non-MIDI or unsupported platform does not receive a
  MIDI selection handle.
- Treat this as active pre-release API design. No compatibility shim for the unrestricted engine
  accessor is required once all repository callers have migrated.

## Testing Decisions

- Tests observe behavior through public boundaries rather than inspecting the handle's private
  engine reference or synchronization state.
- The primary behavioral test constructs a running Orcvs with a fake MIDI backend, obtains the
  restricted handle, discovers and selects a destination, drives Playback through Orcvs, and
  verifies that output reaches the selected fake connection. This proves that configuration and
  Playback share the same adapter and connection.
- Existing shell MIDI tests remain prior art for discovery, selected identity, discovery failure,
  connection failure, and visible status behavior.
- A compile-fail doctest documents and enforces that the MIDI selection handle has no start, stop,
  disconnect, or observe methods.
- Existing Playback lifecycle, cancellation, shutdown, diagnostic, native, persistence, and WASM
  tests remain regression coverage and must continue to pass.
- Native and WASM feature combinations are exercised according to the repository verification
  contract.

## Out of Scope

- Redesigning MIDI destination identity or display names.
- Moving the shell-owned destination list or status into the Orcvs crate.
- Changing MIDI discovery, connection, reselection, or all-notes-off behavior.
- Changing Playback scheduling, cancellation, shutdown, diagnostic ordering, or Source execution.
- Splitting Source, Playback, or MIDI into additional crates.
- Generalizing the handle into an open-ended capability framework.
- Adding MIDI device selection to WASM or another unsupported platform.
- Preserving the unrestricted Playback Engine accessor as a deprecated compatibility API.

## Further Notes

ADR 0022 assigns Playback and MIDI output to the Orcvs crate while assigning MIDI device selection
to the shell. The restricted handle is the narrow coordination seam between those responsibilities:
the shell chooses an output destination, while the running Orcvs remains the sole lifecycle owner.
This work hardens an already-correct implementation boundary; it does not address a current user-
visible correctness defect.
