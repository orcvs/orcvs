# 31 — Write a Source File through the SourceBuffer's checked text

**What to build:** Restore one whole-buffer checked text conversion for `file::write`, then slice its ASCII rows without re-validating each row. Keep the format, signature, allocation bound and public borrowed-bytes accessor unchanged. The paired CI comparison below supersedes the earlier per-row performance rationale.

**Blocked by:** None — 08 is resolved (orcvs/orcvs#165).

**Status:** ready-for-agent

- [ ] `file::write` borrows the whole buffer through one checked text conversion, slices rows at ASCII boundaries, and introduces no `unsafe` or whole-Cell copy. Implement and verify this follow-up against the current source.
- [ ] `Source::cells()` stays the public borrowed-bytes accessor; the console's unsaved-changes check keeps comparing bytes through it; `reading_the_cells_to_compare_them_allocates_nothing` passes.
- [ ] The Source File format and `file::write`'s signature are unchanged; its doctest and the existing Source File round-trip tests pass unmodified under `cargo nextest run --package orcvs` and `cargo test --workspace --doc`.
- [ ] `orcvs/tests/allocation.rs` still holds writing a Source File to less than one copy of the Cells; `writing_a_source_file_allocates_less_than_one_copy_of_the_cells` passes unmodified.
- [x] The current writer is compared with the single-conversion alternative and the measured decision is recorded. — paired CI comparison, 2026-09-28 comment below.

## Comments

**2026-09-26 — origin.** Split from 30 during review of orcvs/orcvs#161: `file::write` is the one shipped text consumer of the Cells outside planning and the Language Map. It is off the Tick path, so it lands separately from 30 to keep each change's benchmarks attributable.

**2026-09-27 — resolved.** `file::write` borrows the Cells once as text through `Source::text`, which carries `SourceBuffer::as_str`'s single check, and slices each row from it; ASCII Cells make every row boundary a character boundary, so no row is re-checked. Gates: fmt, clippy and nextest for `orcvs` and `console`, doctests, the no-default-features suite and `check_wasm` pass.

**2026-09-27 — benchmarked on #174.** Benchmark run 36320645197: `source_file/write/256x256` measured 5,483 ns on the first attempt and 2,370 ns on the second, against 4,506 ns on `main` (run 36313671499); `source_file/read` moved with it (4.79 ms and 3.30 ms against 4.92 ms), so the spread is the runner's. No `source_file` alert fired on either attempt. The first attempt failed on `source_read_revision/128x128` (18 to 77 ns), which this change does not reach; it measured 30 ns on the second attempt, which passed.

**2026-09-28 — reopened on paired benchmark evidence.** `bb7e859b` (#176) replaced #174's single whole-buffer conversion with `Cells::rows` / `Cells::as_str` and removed `Source::text`, so the resolution above describes a superseded implementation. The two writers were compared with an identical harness on empty, sparse and populated 256×256 fixtures. Local runs on one Apple M2 disagreed between two builds that differ only in Cargo.lock (populated: current 6.9% faster, then candidate 28.2% faster). Three paired Linux CI jobs on pinned `0d1245ce` all passed the predeclared decision rule: populated candidate/current ratios 0.7929, 0.7345 and 0.8095; all twelve adjacent populated pairs favored the candidate; no empty/sparse median exceeded 1.05. The [CI report](../benchmarks/31-source-file-write/ci-report.md), [decision rule](../benchmarks/31-source-file-write/ci-plan.md) and [experiment patch](../benchmarks/31-source-file-write/benchmark.patch) preserve the comparison. Restore the single whole-buffer conversion in a focused follow-up and rerun the implementation gates.
