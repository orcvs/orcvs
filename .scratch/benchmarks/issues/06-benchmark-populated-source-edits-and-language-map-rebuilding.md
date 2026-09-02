# 06 — Benchmark populated Source edits and Language Map rebuilding

**What to build:** Extend the Source performance coverage so maintainers can distinguish the cost of applying an edit and rebuilding Source-derived semantic state from the cost of reading and rendering an unchanged revision.

**Blocked by:** 05 — Benchmark populated Source revision reads and Render Frames.

**Status:** resolved

- [x] Criterion measures a valid Cell edit in populated Sources of representative sizes, including all derived Language Map work performed by the public Source edit path.
- [x] Criterion measures an edit representative of incomplete or invalid live Source without including fixture construction in the measured iteration.
- [x] Each measured iteration restores an equivalent starting revision so repeated edits do not silently become no-ops or benchmark progressively different programs.
- [x] Benchmark identifiers clearly distinguish edit and Language Map rebuild measurements from unchanged revision reads and Render Frame derivation.
- [x] Results expose scaling across Source sizes and use black-boxing or equivalent measures to prevent optimization from discarding observed work.
- [x] The standard local and CI benchmark path emits and retains the new measurements.
- [x] The scoped Rust gates and `mise run bench` pass.

## Comments

`source_edit_rebuild_valid` and `source_edit_rebuild_invalid` both rewrite one operand digit of the first Expression on the second row, through `SourceCommander::set` — the same public path a keystroke takes, so the measurement includes the whole Language Map rebuild the edit forces. The valid edit writes another digit the operand accepts; the invalid one writes a character no operand accepts, which is what Source looks like for most of the keystrokes that produce a valid revision.

The restore is the `iter_batched` setup rather than part of the routine, so it is not measured. `BatchSize::PerIteration` is what keeps setup and routine alternating: any larger batch runs every setup before the first routine, so all but the first edit would write a Cell that already held the edited character and would measure a no-op. That failure is silent — the benchmark still reports a number — which is why the batch size is named rather than left at the default.

Naming separates the two kinds of measurement in the stored series: `source_read_revision` and `source_render_frame` read an unchanged revision, `source_edit_rebuild_*` pay for a new one.

The measurements are the point of the issue. An edit costs 31 µs at 16x16, 183 µs at 32x32, and 1.77 ms at 64x64 — roughly quadratic in the Cell count, against a revision read that stays under 100 ns and a Render Frame that stays under 25 µs. The valid and invalid edits cost the same, so the rebuild, not the parse outcome, is where the time goes. Nothing here is a claim about what should change; it is the number a future change has to be measured against.
