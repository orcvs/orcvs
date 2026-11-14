# 02 — Repoint stale citations in code comments

**Status:** needs-triage

**What to do:** These comments cite wording or ADRs that no longer say what the comment claims.
Repoint each to the owning requirement rule or the current ADR:

- `orcvs/src/source/tick/halt.rs:91` quotes Halt wording `CONTEXT.md` no longer has (EVAL-053).
- `orcvs/src/source/portal.rs:15` quotes Portal wording `CONTEXT.md` no longer has (EVAL-030).
- `orcvs/src/grid.rs:985` quotes Grid and Position wording `CONTEXT.md` no longer has (LANG-020,
  LANG-030).
- `orcvs/src/source/language_map.rs:2009`, `:2231` place the row-partition rule in `CONTEXT.md`;
  it is ADRs 0018 and 0033 (LANG-060).
- `lang/src/atom.rs:482`, `:1079` and `lang/src/lib.rs:356` cite ADR 0029, which ADR 0063
  superseded; the rule holds through ADRs 0028 and 0061 (LANG-250).

- `orcvs/src/playback.rs:3142` (a test comment) cites the Playback Engine entry's promise that a
  refused submission leaves the schedule standing; that sentence now points to PLAY-260, where the
  retry is listed as differing from ADR 0016. Repoint the comment to PLAY-260.

**Done when:** every listed comment cites a source that states what it says; crate gates pass.
