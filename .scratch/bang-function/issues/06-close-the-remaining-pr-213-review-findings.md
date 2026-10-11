# 06 — Close the remaining PR #213 review findings

Status: ready-for-agent
Blocked by: 01, 02, 03, 04

**What to build:**

The review findings on PR #213 that the Bang Function does not remove. Ticket 01 removes the provenance findings (anchor mismatch, one-Tick expiry, test helpers planning without the display, the five-argument clump, duplicate roots in the typed-Bang loop); confirm each is gone and close the rest.

## Acceptance criteria

- [ ] A test pins that a Push whose answer is a Bang writes `**` over its complete destination and activates the roots aligned with it, as the Copy, Read, Track and Write tests do.
- [ ] The Bang-root alignment doc no longer claims every caller skips intrinsically active roots.
- [ ] The vertical-Copy test comment no longer says a typed `**` is not a Bang a Copy can carry.
- [ ] Whether a Function answers only Bang is a column of the Function declaration table, not a separate list, and the whole-list test reads it from there.
- [ ] A Function reference example can again state that it writes no Cell.
- [ ] Each provenance finding from the review is confirmed absent after ticket 01, with the test or code fact that shows it.

- [ ] Tickets 01–04 pass together before the feature is marked complete; ADR 0072 lists every Orca departure in the map.
- [ ] `lang/bang-function-prototype.html` is marked non-authoritative or removed.
