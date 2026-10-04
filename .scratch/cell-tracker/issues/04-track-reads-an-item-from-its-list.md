# 04 — Track reads an item from its List

Status: needs-triage
Blocked by: 01, 02

## Work

Implement Track and Lists from
[ADR 0063](../../../docs/adr/0063-a-list-is-cells-in-a-claim-not-a-value.md).
`@t index count` claims `count` two-Cell items east of its operands, read from
the Source at parse. Each Tick it answers item `index % count`: its two
characters, or a blank.

## Acceptance

- [ ] The Parser claims the List as part of Track's Expression; the Language
      Map and Source Paint show its items as data owned by Track.
- [ ] A Clock-driven Track answers the expected item at Tick 0, holds it for the
      Clock's rate, wraps at `count`, and keeps cycling past Tick 255.
- [ ] An index past `count` wraps; a count of `00` diagnoses.
- [ ] A blank item writes spaces south, and Timed Play receiving it emits
      nothing and raises no diagnostic.
- [ ] A Portal write to the count changes the claim from the next Tick, not the
      current one.
- [ ] A writer that changes the index in the same Tick is read in dependency
      order, whether it sits before or after Track in Grid order.
- [ ] Track nests: `!~0064@t0208C4D4E4  G4C5  E404` plays the selected item.
- [ ] Unit and property tests cover single-item and all-blank Lists, malformed
      items (diagnosed where they are played), and claims at the row edge.
- [ ] `CONTEXT.md`, the Function table and the Function reference include Track.
