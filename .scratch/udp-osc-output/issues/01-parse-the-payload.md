# 01 — Parse the Payload of `!u` and `!o`

Status: ready-for-agent
Type: task
Blocked by: None — can start immediately

**What to build:** The Parser recognises `!u` and `!o` as Terminal Output Functions whose claim is
the spelling, the Payload and its closing `!` ([spec](../spec.md), ADR 0086). Neither sends
anything yet.

- [ ] `!u` and `!o` parse wherever a Terminal Output root can start, with the Payload running to the
      next `!` on the row or to the row's end. `!u!` parses with an empty Payload.
- [ ] The Payload's Cells are never parsed. Only the spelling and the closer are occupied Cells, so
      a Write writes into the Payload and a Read reads from it, as they do a Comment's text.
- [ ] The schedule treats every Payload Cell as a read at the Function's Turn, so a Write into the
      Payload lands in the same Tick.
- [ ] A Write that overwrites the closer changes the extent from the next parse.
- [ ] Both are invalid where a value is required, as every Terminal Output Function is.
- [ ] Paint shows the spelling and closer as the Function and the Payload as its text.
- [ ] A `!` a Write puts inside a Payload is read as a character for the rest of that Tick and
      closes the Payload from the next parse.
- [ ] Cells after an early closer parse as ordinary Source, so `!uhi!c000140` diagnoses `c000140`
      as invalid Source.
- [ ] Tests cover a closed, an unclosed and an empty Payload, `!uhi!c000140`, a Write into a
      Payload, and a Write of `!` into one.
