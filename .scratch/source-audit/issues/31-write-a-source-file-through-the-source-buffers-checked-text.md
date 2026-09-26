# 31 — Write a Source File through the SourceBuffer's checked text

**What to build:** `file::write` reads the Cells as text through the `SourceBuffer`'s one checked conversion instead of re-establishing their invariant row by row. It reads `Source::cells() -> &[u8]`, which does not carry the invariant, so it calls `from_utf8(...).expect("a Source row is printable ASCII")` on every row it keeps. `SourceBuffer::as_str` already proves the whole buffer is text once; slicing that `&str` at row boundaries cannot fail, because every Cell is one ASCII byte.

**Blocked by:** 08.

**Status:** ready-for-agent

- [ ] `file::write` reads the Cells through a crate-private text accessor backed by `SourceBuffer::as_str`, and its per-row `from_utf8(...).expect` is gone. No `unsafe` is introduced.
- [ ] `Source::cells()` stays the public borrowed-bytes accessor; the console's unsaved-changes check keeps comparing bytes through it.
- [ ] The Source File format and `file::write`'s signature are unchanged; its doctest and the existing Source File round-trip tests pass unmodified.
- [ ] `orcvs/tests/allocation.rs` still holds writing a Source File to less than one copy of the Cells.
- [ ] The `source_file` benchmark series shows no regression beyond noise, or the ticket records the measured cost.

## Comments

**2026-09-26 — origin.** Split from 30 during review of orcvs/orcvs#161: `file::write` is the one shipped text consumer of the Cells outside planning and the Language Map. It is off the Tick path and moves the `source_file` series rather than 30's Tick and Language Map series, so it lands separately to keep each change's benchmarks attributable.
