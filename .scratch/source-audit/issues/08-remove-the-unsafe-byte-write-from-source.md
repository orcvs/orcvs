# 08 — Evaluate safe Source storage and remove the unsafe byte write

**What to build:** Evaluate a safe replacement for the only `unsafe` block in shipped code (`Source::set_source`, `as_bytes_mut`, in `orcvs/src/source/model.rs`). The block is sound today. This is a measured simplification, not a correctness repair: compare safe alternatives (a Cell-content buffer converted on snapshot, or a checked in-place replace) before deciding whether to remove it. Untrusted bytes reach the write from deserialization and, since 545ea3a1, from reading a Source File (`source::file::read` → `Source::write_cells`).

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Record the measured decision: remove the shipped unsafe block if the safe alternative meets the benchmark criterion below, or retain it with the measured cost and invariant justification. The `GlobalAlloc` counters in `orcvs/tests/allocation.rs` and `lang/tests/allocation.rs` are test-only and out of scope.
- [x] Source still holds exactly one printable ASCII byte per Cell, and construction, deserialization and Source File read still enforce it.
- [x] If removed, the `SAFETY:` comment goes with the block; if retained, its invariant remains explicit. Correct the comment in `orcvs/tests/allocation.rs` naming "the workspace's one `unsafe` block" to distinguish shipped code from test allocators.
- [x] The Source benchmarks (`source_edit_rebuild_*`, `source_whole_grid/edit_rebuild_valid`, `source_file/read/256x256`) show no regression beyond noise, or the ticket records the measured cost and keeps the block with that justification.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Still the only shipped `unsafe`. Criterion 1 could not be met as written because of the test allocators; scoped to shipped code. Source File read added as an entry point, and the benchmarks named.

**2026-09-25 — acceptance-criteria review against `199c3331`.** Removed the unsupported cost claim and reconciled removal with the existing measured-retention alternative.

**2026-09-25 — implementation (epic PR 10).** Decided on branch `perf/safe-source-storage`, stacked on orcvs/orcvs#151 (`c3e2b071`). The shipped `unsafe` block is removed.

- *Candidates.* **A — checked in-place replace:** `inner` stays a `String` and `set_source` calls `String::replace_range(at..=at, …)` with the `CellContent`'s one-character encoding. The range's two character-boundary checks are the only added work, and a broken invariant panics instead of being undefined behaviour. **B — Cell-content buffer converted on snapshot:** `inner: Vec<u8>` written by index, with `snapshot`, `Display` and serialization validating the buffer through `String::from_utf8`/`str::from_utf8`. A third shape, `Box<[CellContent]>`, was not measured: planning and the Language Map read `&[u8]`, so it needs either `unsafe` to reinterpret the slice or a copy per rebuild.
- *Method.* `cargo bench -p orcvs --bench source --locked -- --save-baseline <variant><round> 'source_edit_rebuild_|source_whole_grid/(edit_rebuild_valid|snapshot)|source_file/read/256x256'`, run locally on one machine (Apple M2, 8 cores, other work running: load average 5–9), because this ticket's decision needs a same-machine comparison that the CI gate does not produce. Variants alternated in rounds (`unsafe`, A, B, repeated) so drift in load hits all three; rounds 1–2 cover every named benchmark plus `source_whole_grid/snapshot`, rounds 3–5 repeat `source_file/read/256x256` and `source_edit_rebuild_valid/64x64`. Figures are criterion medians read from `target/criterion/**/<baseline>/estimates.json`; ranges are the spread across rounds.

  | Benchmark | `unsafe` (baseline) | A `replace_range` | B `Vec<u8>` |
  |---|---|---|---|
  | `source_edit_rebuild_valid/16x16` | 527–541 ns | 540–580 ns | 538–541 ns |
  | `source_edit_rebuild_valid/32x32` | 1.250–1.304 µs | 1.312–1.317 µs | 1.308–1.344 µs |
  | `source_edit_rebuild_valid/64x64` (5 rounds, median) | 2.640 µs | 2.667 µs | 2.660 µs |
  | `source_edit_rebuild_invalid/16x16` | 680–697 ns | 702–703 ns | 699–702 ns |
  | `source_edit_rebuild_invalid/32x32` | 1.445–1.464 µs | 1.447–1.450 µs | 1.443–1.451 µs |
  | `source_edit_rebuild_invalid/64x64` | 2.875–2.905 µs | 2.839–2.922 µs | 2.833–2.853 µs |
  | `source_whole_grid/edit_rebuild_valid` | 9.84–10.21 µs | 10.04–10.05 µs | 10.07–10.23 µs |
  | `source_file/read/256x256` (5 rounds, median; range) | 3.346 ms (3.180–3.431) | 3.447 ms (3.395–3.658) | 3.358 ms (3.273–7.258) |
  | `source_whole_grid/snapshot` (not named here; checked for B) | 1.20–1.22 µs | 1.21 µs | 3.13–3.16 µs |

- *Reading.* Every edit benchmark, including the shipped-shape one, is within the baseline's own run-to-run spread for both candidates: the byte write is a few nanoseconds under a row rebuild of hundreds of nanoseconds to microseconds. `source_file/read/256x256` is the one path that writes many Cells in one revision (every non-space Cell of the fixture through `write_cells`). There A's median sits about 3% (~0.1 ms) above the baseline's, and was higher in each of the five paired rounds, so it is probably a real per-write cost of a few nanoseconds rather than pure noise; it is still inside the baseline's own 3.18–3.43 ms spread across rounds, and CI's benchmark gate on the PR is the authority on whether it alerts. B leaves read and edits unchanged but makes `snapshot` 2.6× slower (~+1.9 µs of UTF-8 validation over 64 KiB), and every `str` view of the Source (snapshot, `Display`, serialization, Source File write) pays that validation.
- *Decision — A, remove the block.* A matches the baseline on every edit path and costs at most the small read difference above; B trades that for a validation on every text view and a wider change. A keeps the storage type, so `snapshot`, `Display`, serialization, planning and the Language Map are untouched. `CellContent::byte` lost its last caller and is removed. The `SAFETY:` comment went with the block; the storage invariant is now stated on `Source::inner`, and `set_source` states why the replace keeps length and Cell indices.
- *Invariant enforcement, unchanged paths, with tests.* Construction: `Source::new` fills spaces; `set` refuses anything but one printable ASCII byte (`test_set_rejects_invalid_content_without_mutation`); `write_cells` takes `CellContent` only. Deserialization: new `test_source_deserialization_refuses_every_byte_that_is_not_a_cell` feeds tab, LF, CR, NUL, DEL, `é` and `☃` padded so the byte count still matches the Grid, and fails when the content check is removed (confirmed by temporarily deleting it). Source File read: `reading_refuses_every_byte_that_is_not_printable_ascii_or_space` and `reading_refuses_each_class_of_byte_that_is_not_a_cell` already cover every byte.
- *Comments and tooling text.* `orcvs/tests/allocation.rs` names the checked `replace_range` instead of "the workspace's one `unsafe` block". `docs/tooling.md`, the `miri` task comment in `mise.toml` and `.github/workflows/miri.yml` now say shipped code holds no `unsafe` block, and that the workspace's own `unsafe` lives only in allocators test builds install (worded so a further `cfg(test)` allocator keeps it true). The Miri task and its filter are unchanged. `AGENTS.md`'s Miri bullet still names "the byte write in `Source::set_source`"; it is left for a human edit.
- *Miri.* Not run: no `unsafe` remains in the changed code, and the task is manual-dispatch only.
