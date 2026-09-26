# 08 — Hold the Cells in a shared SourceBuffer and remove the unsafe byte write

**What to build:** The Source holds its Cells in a `SourceBuffer` (the type of `Source.inner`): one printable ASCII byte per Cell, in Grid order, shared read-only with every planning snapshot and Source revision taken from it, and copied only when the Source writes while such a reader still holds it. Before this ticket the Cells were a `String`, and three things followed from that. The only `unsafe` block in shipped code (`Source::set_source`, `as_bytes_mut`) existed because a `String` offers no safe single-byte write. Each optimistic Tick attempt copied every Cell into its planning snapshot, on top of the working copy Tick execution already makes. And every Render Frame's `read_revision` copied every Cell again — 64 KiB each on the shipped Grid. Untrusted bytes reach the Cells from deserialization and from reading a Source File (`source::file::read` → `Source::write_cells`); both still enforce the invariant.

**Blocked by:** 28 (the measured Tick baseline this is compared against).

**Status:** resolved

- [x] `Source.inner` is a private `SourceBuffer` over shared bytes. Every write takes a `CellContent` and goes through safe copy-on-write, so the shipped `unsafe` block and its `SAFETY:` comment are gone. Correct the comment in `orcvs/tests/allocation.rs` naming "the workspace's one `unsafe` block" to distinguish shipped code from the test allocators, which stay out of scope.
- [x] Source still holds exactly one printable ASCII byte per Cell, and construction, deserialization and Source File read still enforce it.
- [x] A planning snapshot and a Source revision share the Source's buffer rather than copying it; a test pins the sharing by pointer identity, and one pins that a write while a reader holds the buffer leaves that reader's Cells unchanged. `PlanningSnapshot` and `SourceRevision` remain separate types.
- [x] An edit that lands while a snapshot or revision holds the buffer copies it under the write lock. That cost is accepted, and measured.
- [x] Tick planning, the Language Map and Claim lookup keep taking `&[u8]`, borrowed from the buffer; passing the buffer's type through them is 30.
- [x] `snapshot() -> String` remains the owned, copying form.
- [x] The persisted format is unchanged, and the existing format test passes unmodified. Loading converts the stored text into the buffer once; saving borrows the Cells as text through a checked view instead of cloning them. The Language Map built twice on load is out of scope.
- [x] Measured against 28's baseline: 28's Tick series and allocation record, `source_read_revision`, `source_whole_grid/snapshot`, `source_edit_rebuild_*`, `source_whole_grid/edit_rebuild_valid` and `source_file/read/256x256`. No regression beyond noise, or the measured cost is recorded here with a decision.
- [x] The decision is recorded in a comment on this ticket, with a brief comparison against a checked in-place replace on a `String`. ADR 0057's closing cost paragraph, which says each attempt copies the Cell bytes, is amended to the new cost.
- [x] The `rust-unsafe` gate applies to the removal. Miri is optional (`mise run miri`); it interprets the Source model tests this change touches.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Still the only shipped `unsafe`. Criterion 1 could not be met as written because of the test allocators; scoped to shipped code. Source File read added as an entry point, and the benchmarks named.

**2026-09-25 — acceptance-criteria review against `199c3331`.** Removed the unsupported cost claim and reconciled removal with the existing measured-retention alternative.

**2026-09-26 — rescoped to a shared SourceBuffer.** Decided in the architecture review of epic PR 9 (`perf/plan-outside-lock`). Planning outside the Source lock (07) added a whole-Cell copy per optimistic Tick attempt, beside the one `read_revision` makes per Render Frame; a buffer shared by the Source and its readers removes both and the `unsafe` byte write with them, which makes it the safe alternative this ticket evaluates rather than a third option beside it. The measured-retention outcome is no longer offered: the shared buffer is the design, and the benchmarks decide only whether its cost is recorded. Decisions:
- *Representation.* A private newtype over shared bytes, written through `CellContent`. A buffer of `CellContent` was rejected: every consumer takes `&[u8]`, and viewing `[CellContent]` as bytes needs an unsafe cast or a new dependency. A shared `String` with a checked replace was rejected for double indirection and splice cost on every edit.
- *Name.* `SourceBuffer`, as the type of `Source.inner` only. It is not a `CONTEXT.md` term: the Source entry avoids "buffer", and the type does not leave the Source module until 30.
- *Split out.* Borrowed reads for Source File write and the console's unsaved-changes check are 29. Passing the buffer's type through planning and the Language Map is 30. Merging `PlanningSnapshot` into `SourceRevision` is not scheduled.

**2026-09-26 — decision (orcvs/orcvs#165).** Removed. `Source.inner` is a private `SourceBuffer` over `Arc<[u8]>`, written through `CellContent` via `Arc::make_mut`; the shipped `unsafe` block and its `SAFETY:` comment are gone. Planning snapshots and Source revisions clone the `Arc`; tests pin sharing by pointer identity and a reader's Cells surviving a later write.

Measured against 28's baseline (Apple M-series, loaded machine, alternating rounds; allocations exact, timings indicative): the Playback Tick's extra 64 KiB block is gone — a settled Tick on the shipped Grid allocates 35,185 blocks / 11,064,920 bytes on both paths, and the allocation test now holds the Playback Tick to the locked Tick. `source_read_revision/256x256` falls from ~1.3 µs to ~16 ns, flat across sizes. Edit, Source File and Tick series are within noise. `source_whole_grid/snapshot` rises from ~1.3 µs to ~3.4 µs: safe code must UTF-8-validate the bytes to view them as `&str`. Accepted: after 29, `snapshot()`'s shipped callers are startup, open and the Source File save baseline, none per Tick or per Render Frame. Avoiding the check needs `unsafe` or an ASCII-typed buffer (unstable `ascii::Char`, or a dependency).

Against the checked in-place replace (#155, `String::replace_range` over one byte): it removes the `unsafe` block at no snapshot cost, but keeps the `String`, so every planning snapshot and every `read_revision` still copies the Cells — the per-Tick and per-frame copies this ticket targets.

ADR 0057's cost paragraph is amended.
