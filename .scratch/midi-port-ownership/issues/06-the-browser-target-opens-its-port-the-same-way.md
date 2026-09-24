# 06 — The browser target opens its port the same way

**What to build:** A performer running Orcvs in the browser gets the same MIDI menu behaviour as on
the desktop: destinations discovered on the main thread and shown in the frame that asked, a port
opened there, and Playback delivering to it through the same open connection handed across the same
ordered queue. One description of MIDI selection covers both platforms, and the browser path stops
being the one that has to be reasoned about separately.

**Blocked by:** None — 03 landed; the architectural shape is in place.

**Status:** ready-for-agent

**Tags:** release/v1

## Already landed

The console owns the backend on every target, calls discovery and connect synchronously, and queues
an open connection through `install` on the ordered Playback queue. The WASM build compiles with
that shape. What remains is wiring the Web MIDI API so the browser backend is functional rather
than a stub that lists nothing and refuses every connect.

- [x] The browser delivers an already-open connection to the Playback Engine on the ordered queue;
      the engine's task owns it exactly as it owns the native one.
- [x] Console frame reads stay nonblocking on the browser main thread, which has no blocking
      receive.
- [x] The WASM target compiles and the headless browser suite passes, per the repository
      verification contract.

## Remaining

- [ ] Browser discovery and port opening happen on the main thread through the Web MIDI API and
      answer the caller that asked, matching the native shape.
- [ ] A performer can refresh destinations in the browser, select one, and hear output from it.
- [ ] A build for a target with no MIDI service still falls back to the silent backend and offers an
      empty destination list rather than an error.

## Comments

**2026-09-24 — joins `release/v1`.** The release decision is to build Web MIDI: the WASM build promises MIDI output, as the release goal ("deterministic native and WASM playback") and ADR 0041 state. This issue blocks `v1-release/03`. Proof is fake-adapter tests of exact bytes and lifecycle on the browser path; physical-device evidence stays native-only under `v1-release/04`. Until this lands, the web console's MIDI list and Refresh are offered while `AVAILABLE` is true on wasm but can never find a destination.
