# 05 — Diagnose `!u` and `!o` in the web build

Status: resolved
Type: task
Blocked by: 02, 03

**What to build:** In the web console, `!u` and `!o` parse and paint as natively, and an activated
one reports that this build cannot send UDP or OSC ([spec](../spec.md), ADR 0085).

- [x] The diagnostic appears once per activation, and playback continues.
- [x] Nothing in the web build opens a socket.
- [x] `mise run check_wasm` passes.

## Answer

The web build's network output has no socket: every send answers the failure "this build cannot
send UDP or OSC", so each activation is one `NetworkFailure` diagnostic. The console reports each
to the developer console and shows the last beside the MIDI status. The browser suite in
`console/tests/wasm.rs` pins one diagnostic per activation while playback goes on. The clippy step
of `mise run check_wasm` passes for both feature sets; its `trunk` builds were left to CI.
