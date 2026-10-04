# 04 — Track reads current-Tick Items from its List

Status: ready-for-agent
Blocked by: 02, 03

**What to build:**

Implement ADR 0063’s Track from parsing through Source/Tick delivery and
Source Paint. Establish List structure once per Tick, then select live Item
characters after their suppliers settle. A performer can hear same-Tick edits
made by other Functions and can nest Track without changing literal meaning.

## Acceptance criteria

- [ ] The Parser claims the literal count and every two-Cell Item as Track data; nested count Functions are refused and Item spellings never execute as Functions or open Comments inside the claim.
- [ ] Clock-driven Track selects the expected Item at Tick zero, holds it for the Clock rate, wraps at the count and continues beyond Tick 255.
- [ ] The parsed count governs claim ownership, modulo selection and zero validation for the whole Tick. A same-Tick count write takes effect for all three only on the next Tick.
- [ ] Tests grow count from two to three while index is two, shrink it and set it to zero; assert current- and next-Tick selection and claim, and no read beyond the established claim.
- [ ] The complete possible Item read extent participates in dependency discovery. Writers to the index and selected Item are observed in the same Tick whether they precede or follow Track in Grid order, including partial and competing writes and failed suppliers.
- [ ] Partial and competing Item writes compose Cell-wise in dependency order. Absent and failed suppliers leave surviving Source characters available to Track.
- [ ] Read/write cycles preserve atomic publication and publish no partial writes or Play Commands; cover overlap with declared Item reads.
- [ ] Structural count edits produce the same observable result with reused scheduling and freshly derived scheduling.
- [ ] Track copies a blank Item as two spaces south, clearing a previous Note. The receiving Timed Play emits no command or diagnostic. A nested parent with another value operand answers blank and clears its own Output Portal; it does not preserve that destination.
- [ ] Nested Track supplies its selected encoding to the parent and also writes its own Output Portal. A blank Item clears Track’s south Cells while the parent answers blank and clears its own Output Portal.
- [ ] Contrast blank Item delivery with the Absence Marker that ticket 03 keeps as a no-write result: no-write absence must not be used as the representation of an explicit clear.
- [ ] Cover single-Item and all-blank Lists, malformed selected and unselected Items, partly empty Items, invalid counts and claims at the row edge. Malformed selected data diagnoses at its receiving operand.
- [ ] Record frozen-count and live-Item-read semantics in ADR 0063; the glossary, Function table, Function reference and Source Paint agree with Track’s behavior.
