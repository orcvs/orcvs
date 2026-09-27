# 31 — Write a Source File through the SourceBuffer's checked text

**What to build:** `file::write` reads the Cells as text through the `SourceBuffer`'s one checked conversion instead of re-establishing their invariant row by row. It reads `Source::cells() -> &[u8]`, which does not carry the invariant, so it calls `from_utf8(...).expect("a Source row is printable ASCII")` on every row it keeps. `SourceBuffer::as_str` already proves the whole buffer is text once; slicing that `&str` at row boundaries cannot fail, because every Cell is one ASCII byte.

**Blocked by:** None — 08 is resolved (orcvs/orcvs#165).

**Status:** resolved

- [x] `file::write` reads the Cells through a crate-private text accessor backed by `SourceBuffer::as_str`, and its per-row `from_utf8(...).expect` is gone. No `unsafe` is introduced. — `Source::text` (`pub(super)`, `orcvs/src/source/model.rs`) returns `SourceBuffer::as_str`; `write` slices it per row and trims with `str::trim_end_matches`.
- [x] `Source::cells()` stays the public borrowed-bytes accessor; the console's unsaved-changes check keeps comparing bytes through it. — unchanged; `reading_the_cells_to_compare_them_allocates_nothing` passes.
- [x] The Source File format and `file::write`'s signature are unchanged; its doctest and the existing Source File round-trip tests pass unmodified. — no test or doctest edited; `cargo nextest run --package orcvs` (680 passed) and `cargo test --workspace --doc` pass.
- [x] `orcvs/tests/allocation.rs` still holds writing a Source File to less than one copy of the Cells. — `writing_a_source_file_allocates_less_than_one_copy_of_the_cells` passes unmodified (two blocks: the row list and the text).
- [x] The `source_file` benchmark series shows no regression beyond noise, or the ticket records the measured cost. — judged by the pull request's Benchmark workflow, which compares the `source_file` series against `main`; no local comparison was run (`.scratch/benchmarks/spec.md`).

## Comments

**2026-09-26 — origin.** Split from 30 during review of orcvs/orcvs#161: `file::write` is the one shipped text consumer of the Cells outside planning and the Language Map. It is off the Tick path, so it lands separately from 30 to keep each change's benchmarks attributable.

**2026-09-27 — resolved.** `file::write` borrows the Cells once as text through `Source::text`, which carries `SourceBuffer::as_str`'s single check, and slices each row from it; ASCII Cells make every row boundary a character boundary, so no row is re-checked. Gates: fmt, clippy and nextest for `orcvs` and `console`, doctests, the no-default-features suite and `check_wasm` pass.
