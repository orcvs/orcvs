# 08 — Give roles a border of their own

**What to build:** ADR 0078 ranks "the role border of a Cell its role claims" above the Grid line, and ticket 03 ranks every other border fact, but no Theme has a role border: a Function, Number or Note Cell gets a fill and a Glyph colour from its role and no border. Decide whether roles get border keys (a colour and a width per role, transparent by default, with schema, resolver and built-in Theme changes), and if so add them as `EdgeRank::Role` between the Grid line and Diagnostic. Until then the rank stays in the ADR with no facts in it. Toby chose to ship 03 without role borders (2026-10-21).

**Blocked by:** 03 — Draw every border edge once.

**Status:** needs-triage
