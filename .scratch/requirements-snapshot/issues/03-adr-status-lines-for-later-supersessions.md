# 03 — Status lines that do not name a later superseding ADR

**Status:** needs-triage

**What to decide:** whether the ADR process allows adding a forward pointer to an accepted ADR's
Status line when a later ADR supersedes part of it. `docs/adr/README.md` does not say. Each later
ADR below already states what it replaces, so the requirements follow it (decisions D7):

- 0005 → 0070 (the `&` prefix and directional Address Functions).
- 0006 → 0071 (direct Portal delivery of a Bang).
- 0014 → 0072 (contact activation).
- 0017 → 0070 (the `@<`/`@>` reservation).
- 0001 → 0016 ("all-notes-off" widened to the full safety action).
- 0053 → its own 2026-10-31 amendment (no control returns to following the OS appearance).

Readers also found stale facts in ADR bodies that later ADRs changed without a Status line: 0087's
kit channels (0099), 0091's phone Cell size (0102), 0055's failure routing (0092), 0094's
`orca-tracker` link name (the code uses `tracker`).

**Done when:** the ADR process states whether such forward pointers are permitted, and they are
added or the decision not to is recorded.
