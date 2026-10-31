# The Output list shows every output type

Status: accepted. Settled with Toby on 2026-10-29 in `.scratch/output-list/`. [ADR 0096](0096-the-overview-sits-in-the-sources-corner-at-one-pixel-per-cell.md) moves the Overview into the Source's corner, above the Panel, so it no longer covers the status messages; the line beneath the Panel's row stands.

The Panel's **O** is Output, not MIDI. Its list held the MIDI destinations with the UDP and OSC
fields unlabelled at its foot, and its closed field named only the MIDI destination. The Panel's
status messages followed **O** on the Panel's row. At the default 1024-point window that row ends
at x=552, and the Overview, which floats over the window's bottom-right corner, can begin at x=512,
so an open Overview could cover them.

## Decision

**The Output list has one labelled section per output type.** **MIDI** holds Scan and the
destinations; **UDP/OSC** holds the host and the two ports. A separator divides them. A section
this build cannot use is not shown: the web build has no UDP/OSC section, and a build without a
MIDI backend has no MIDI section.

**The closed field names every live output, joined by ` · `.** The selected MIDI destination, then
the network host, so a native console with a synth selected reads `Studio Synth · 127.0.0.1`.
With nothing live it reads `None`. A failure stays in the status messages, never in the field.

**The status messages take a line of their own beneath the Panel's row.** The line spans the
window's width above the Diagnostics readouts and is there only while a message is, whether or not
the readouts are open. It starts at the window's left edge, clear of the Overview.

**`destination_presentation` goes.** With each section shown only where it can be used, the list is
enabled everywhere and Scan belongs to the MIDI section, so nothing is left for it to decide, and
the test-only copy that flipped `console_midi::AVAILABLE` goes with it.

## Considered options

- **The network destination as its own Panel entry after O.** It cannot fit beside **O** at the
  default window without sitting under the Overview.
- **The network destination in the menu bar.** It splits Output across two places.
- **Status messages left after O.** An open Overview covers them.
