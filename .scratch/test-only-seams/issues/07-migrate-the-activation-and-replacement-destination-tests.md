# 07 — Migrate the activation and replacement destination tests

**What to build:** The final migration batch. Move the remaining destination-carrying tests —
Bang activation and its ordering edges, suppression of a nested computation by an inactive parent,
Function replacement at an original anchor, and the diagnostics each of those produces — onto the
constructed schedule from `04`. Their assertions do not change.

When this ticket lands nothing populates the destination map.

**Blocked by:** 04 — Build a destination-carrying schedule a test can construct.

**Status:** ready-for-agent

- [ ] Every remaining destination-carrying test drives execution through the constructed schedule.
- [ ] No test in the crate populates the destination map after this ticket.
- [ ] Every behaviour the batch asserted before it moved is still asserted after.
- [ ] The crate builds and its tests pass.

## Comments

Split out of the architecture review of the Sequence branch, 2026-09-10.
