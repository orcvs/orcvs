# 09 — Decide and build the Application Command `!$`

**Status:** needs-triage

**What to decide:** ADR 0008 accepts Application Command `!$` (LANG-460) and defers its command
value encoding. The Function table has no `!$` row, so `!$` diagnoses as an unknown spelling.
`v1-roadmap-wayfinding/01` defers it past the first release. `docs/requirements/language.md` lists it
under "Accepted but not built".

**Done when:** an ADR states the command value encoding and `!$` is in the Function table, or an ADR
retires LANG-460.
