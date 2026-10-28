# 01 — Carry Network Sends in the Tick Plan

Status: ready-for-agent
Type: task

**What to build:** `!u` and `!o` emit a Network Send into its own ordered list on the Tick Plan, and
`PlayCommand` loses its `Udp` and `Osc` variants ([spec](../spec.md), ADR 0090).

- [ ] Every consumer of Play Commands stops filtering out network variants.
- [ ] `PlayCommand` and `Interpretation` return to their MIDI-only size, with the size assertion
      restored.
- [ ] Tick Plan order holds within the Network Send list.
