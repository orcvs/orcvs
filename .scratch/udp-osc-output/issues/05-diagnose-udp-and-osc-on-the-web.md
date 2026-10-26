# 05 — Diagnose `!u` and `!o` in the web build

Status: ready-for-agent
Type: task
Blocked by: 02, 03

**What to build:** In the web console, `!u` and `!o` parse and paint as natively, and an activated
one reports that this build cannot send UDP or OSC ([spec](../spec.md), ADR 0086).

- [ ] The diagnostic appears once per activation, and playback continues.
- [ ] Nothing in the web build opens a socket.
- [ ] `mise run check_wasm` passes.
