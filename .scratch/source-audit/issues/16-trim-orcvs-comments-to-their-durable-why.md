# 16 — Correct orcvs comments the source-audit changes leave behind

**What to build:** After source-comments/03 holds `orcvs` to `docs/agents/comments.md` (lineage, provenance citations, `SAFETY:`/doctest/intra-doc/`reason =` carve-outs), what remains is the work that rule does not cover and that depends on the code the orcvs source-audit tickets change: each argument stated once, and doc comments no longer than the functions they describe. Comments are 35–47% of shipped lines in the largest modules (`playback`, `app`, `portal`, `tick`, `execution`, `model`, `language_map`).

**Blocked by:** source-comments/03; 01 — Derive the Tick period from BPM without truncation; 02 — Close the Tick gate race; 06 — Reuse unchanged Language Map rows; 07 — Plan a Tick outside the Source write lock; 08 — Remove the unsafe byte write from Source; 10 — Move orcvs test-only state out of shipped code; 19 — Cache the Tick schedule; 20 — Derive Sequence capability in one place; 21 — Bound the Playback diagnostics queue.

**Status:** ready-for-agent

- [ ] Each argument appears once, at the item that enforces it.
- [ ] Doc comments are no longer than what a caller needs to use the item.
- [ ] Concurrency, ordering and invariant statements are kept.
- [ ] Comments that contradict the code after the tickets above land are corrected or removed.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** The lineage and citation criteria duplicated source-comments/03 (filed by 0ce8d2c4, ready-for-agent). The old criterion banning every issue-tracker citation also contradicted rule 3 of `docs/agents/comments.md`, which keeps a citation to an open ticket. Manifest comments belong to source-comments/05. Removed 23 (obsolete) from the blockers.
