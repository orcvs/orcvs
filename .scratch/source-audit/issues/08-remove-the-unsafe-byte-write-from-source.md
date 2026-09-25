# 08 — Remove the unsafe byte write from Source

**What to build:** Evaluate a safe replacement for the only `unsafe` block in shipped code (`Source::set_source`, `as_bytes_mut`, in `orcvs/src/source/model.rs`). The block is sound today. This is a measured simplification, not a correctness repair: compare safe alternatives (a Cell-content buffer converted on snapshot, or a checked in-place replace) before deciding whether to remove it. Untrusted bytes reach the write from deserialization and, since 545ea3a1, from reading a Source File (`source::file::read` → `Source::write_cells`).

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] Record the measured decision: remove the shipped unsafe block if the safe alternative meets the benchmark criterion below, or retain it with the measured cost and invariant justification. The `GlobalAlloc` counters in `orcvs/tests/allocation.rs` and `lang/tests/allocation.rs` are test-only and out of scope.
- [ ] Source still holds exactly one printable ASCII byte per Cell, and construction, deserialization and Source File read still enforce it.
- [ ] If removed, the `SAFETY:` comment goes with the block; if retained, its invariant remains explicit. Correct the comment in `orcvs/tests/allocation.rs` naming "the workspace's one `unsafe` block" to distinguish shipped code from test allocators.
- [ ] The Source benchmarks (`source_edit_rebuild_*`, `source_whole_grid/edit_rebuild_valid`, `source_file/read/256x256`) show no regression beyond noise, or the ticket records the measured cost and keeps the block with that justification.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Still the only shipped `unsafe`. Criterion 1 could not be met as written because of the test allocators; scoped to shipped code. Source File read added as an entry point, and the benchmarks named.

**2026-09-25 — acceptance-criteria review against `199c3331`.** Removed the unsupported cost claim and reconciled removal with the existing measured-retention alternative.
