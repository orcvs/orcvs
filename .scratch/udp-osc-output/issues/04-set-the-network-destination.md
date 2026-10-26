# 04 — Set the UDP and OSC destination

Status: ready-for-agent
Type: task
Blocked by: 02

**What to build:** The native console lets the performer set the host, the UDP port and the OSC port
next to MIDI port selection, and saves them with its other settings ([spec](../spec.md), ADR 0086).

- [ ] Defaults are `127.0.0.1`, UDP `49161` and OSC `49162`.
- [ ] The setting lives in the console, never in Source.
- [ ] A change takes effect at the next send, and an invalid host or port is reported, not saved.
- [ ] The setting round-trips through persistence, and the `--no-default-features` build still
      compiles.
