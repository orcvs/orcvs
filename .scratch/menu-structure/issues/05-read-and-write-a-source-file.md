# 05 — Read and write a Source File

**What to build:** The `orcvs` crate turns a Source into Source File text and Source File text into a Source, per `spec.md`'s format and ADR 0054. No UI.

**Blocked by:** 01 — Fix the Grid at 256 by 256.

**Status:** resolved

**Tags:** release/v1

- [x] Writing produces one LF-terminated line per row up to the last row holding content, each with trailing spaces trimmed.
- [x] Reading accepts LF and CRLF, treats short and missing lines as empty Cells, and places line *n* character *m* at column *m*, row *n*.
- [x] Reading refuses the whole text, naming line and column, for any byte that is not printable ASCII or space (tab included), a 257th line, or a line past 256 characters. A refusal constructs no Source.
- [x] The Function reference loader reads through this, and its ragged-text test still holds.
- [x] A property test: every Source round-trips through write then read unchanged.
- [x] Boundary tests for 256 lines and 256-character lines accepted, 257 refused, each refused byte class, and CRLF.
- [x] Scoped gates for `orcvs` and `console` pass.

## Comments

**Implementation.** `orcvs::source::file` holds `read(&[u8]) -> Result<Source, SourceFileError>` and `write(&Source) -> String`. `read` takes bytes, not a `str`, so a non-UTF-8 file is refused at the offending byte with its line and column. `SourceFileError` carries a one-based `line` and `column` and a `Refusal` (`NotACell(byte)`, `TooManyLines`, `LineTooLong`), and its `Display` names a tab and a lone CR explicitly. A trailing LF ends the last line rather than opening a 257th; an empty 257th line is still refused.

**No new Source constructor.** `read` validates every byte as a `CellContent` and builds the Source with `Source::new` plus one `write_cells`, and `write` reads `snapshot()`. The planned `Source::from_cells`/`cells()` were not added: `write_cells` already writes through `set_source`, whose SAFETY argument relies on every write being a `CellContent`, so the unsafe block and its invariant are untouched and no second construction path has to re-establish them.

**Function reference.** `source_from_reference_text` is now `file::read`, panicking with the refusal (the asset is compiled in, so a refusal is an asset defect). `the_checked_in_reference_is_a_source_file` pins that, and `a_ragged_text_with_stripped_trailing_whitespace_loads_onto_the_one_grid` still holds.

**Tests.** Unit tests cover LF/CRLF, short and missing lines, empty text, 256 lines of 256 characters accepted (with and without the final LF), the 257th line and 257th character refused (trailing spaces count, a CRLF does not), every byte 0x00-0xFF but LF, each refused class, and the `Display` text. The property `every_source_round_trips_through_write_then_read` draws few, many, or whole-row writes with extra weight on the last column, last row and first row.

**Benchmark.** `orcvs/benches/source.rs` group `source_file`: `read/256x256`, `write/256x256`.

**Gates.** fmt; clippy `orcvs` and `console` (native and `wasm32-unknown-unknown`); nextest `orcvs` (`PROPTEST_CASES=32`) and `console`; `orcvs` doctests; `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`; `cargo bench --bench source -- --test source_file`. All passed.
