# 04 — Set the UDP and OSC destination

Status: resolved
Type: task
Blocked by: 02

**What to build:** The native console lets the performer set the host, the UDP port and the OSC port
next to MIDI port selection, and saves them with its other settings ([spec](../spec.md), ADR 0085).

- [x] Defaults are `127.0.0.1`, UDP `49161` and OSC `49162`.
- [x] The setting lives in the console, never in Source.
- [x] A change takes effect at the next send, and an invalid host or port is reported, not saved.
- [x] The setting round-trips through persistence, and the `--no-default-features` build still
      compiles.

## Answer

The host and ports are fields at the foot of the Panel's Output list, beneath the MIDI
destinations. A row of their own on the Panel would sit under the Overview, which covers the
window's bottom-right corner at the default size. The console's storage keeps them under
`orcvs_network_destination`, beside the window geometry, and a refused value is reported beside the
MIDI status.
