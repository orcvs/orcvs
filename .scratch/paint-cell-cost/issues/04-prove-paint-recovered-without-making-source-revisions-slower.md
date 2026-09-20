# 04 — Prove Paint recovered without making Source revisions slower

**What to build:** Prove on the authoritative benchmark runner that `03` removes the dominant Paint
cost without transferring it to Source revision reads, Source edits, Tick execution, or Render
Frame derivation. Record the result and reconcile the decisions and deferred cache work against the
measured implementation.

**Blocked by:** 03 — Remove the Paint claim cache without caching Paint on the Language Map.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] The pull-request Benchmark workflow compares the candidate with the rebased `origin/main` on
      the same runner and records every `paint_derive`, `paint_background_runs`,
      `source_read_revision`, `source_edit_rebuild_valid`, `source_edit_rebuild_invalid`,
      `source_execute_tick`, and `source_render_frame` result.
- [ ] Both fitted and culled `paint_derive` series materially improve in the direction the profile
      in `01` predicts: the pointer-keyed cache and its growth cost are absent from the measured
      path.
- [ ] No Source revision read, valid or invalid edit rebuild, Tick execution, Render Frame, or
      background-run series regresses beyond ordinary runner noise. Any apparent regression is
      rerun and resolved rather than accepted because the Paint series improved.
- [ ] ADR 0050 is amended or superseded to record the measured seam: the Render Frame carries the
      once-derived written answer, while the Language Map remains authoritative for the parser's
      claim and does not retain whole-Grid Paint views.
- [ ] `paint-cell-cost/02` records that its whole-Grid Language Map implementation was rejected by
      the Source benchmark regressions, without erasing the implementation and review history.
- [ ] The remaining claim-cache criterion in `syntax-highlighting/07` is checked only after the
      benchmark comparison proves the cache is gone without transferring unacceptable cost.
- [ ] The final record names the candidate SHA, baseline SHA, workflow run, exact measurements, and
      explicit accept or reject decision.
