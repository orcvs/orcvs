# Output list

Status: resolved

## The decision

The Panel's **O** list shows every output type, and the status messages leave the Overview's corner.
Toby settled this (review candidate 4, revised) on 2026-10-29.
[ADR 0095](../../docs/adr/0095-the-output-list-shows-every-output-type.md) records it.

## Rules

- **Sections.** **MIDI** (Scan, the destinations), a separator, **UDP/OSC** (Host, UDP, OSC). A
  section this build cannot use is not shown.
- **Closed field.** Every live output joined by ` · `: the MIDI destination's name when one is
  selected, then the host on native. `None` when nothing is live.
- **Status line.** The status messages are drawn on a line beneath the Panel's row, from the left
  edge, present only while a message is.
- **No presentation struct.** `DestinationPresentation`, `destination_presentation` and the
  `cfg(test)` copy go; the list is always enabled.

## Tests

The closed field's text is a pure function with its own tests. The console's kittests open **O**
and find both section labels, read the host in the closed field, and find a status message below
the row.

## Delivery

One pull request, one commit per ticket.
