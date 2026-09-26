# 29 — Stop copying the Source to write a Source File or check for unsaved changes

**What to build:** Writing a Source File and the console's unsaved-changes check read the Source's Cells in place instead of copying them. Both call `snapshot()`, which clones every Cell into a new `String` — 64 KiB on the shipped Grid — and then only chunk or compare the bytes. The unsaved-changes check runs under `read_source`, so the copy also lengthens a read guard that a Tick commit waits behind.

**Blocked by:** None — can start immediately. Independent of 08: the accessor reads whatever holds the Cells.

**Status:** resolved

- [x] `Source` answers its Cells as borrowed bytes through one accessor, without allocating. `snapshot() -> String` remains the owned, copying form for callers that keep the text.
- [x] Writing a Source File reads through the accessor. The Source File format is unchanged, and the existing Source File round-trip tests pass unmodified.
- [x] The console's unsaved-changes check compares its saved baseline against the accessor rather than a fresh `snapshot()`. The saved baseline itself stays an owned `String`.
- [x] A test or allocation check shows neither path allocates a copy of the Cells.

## Comments

**2026-09-26 — origin.** Split from 08 during the design of the shared `SourceBuffer`: every shipped reader of the Cells needs only borrowed bytes except the console's saved baseline, and these two readers copy for nothing. Landing it separately keeps 08 about storage.

**2026-09-26 — resolved (orcvs/orcvs#162).** `Source::cells() -> &[u8]` borrows the Cells in Grid order; `snapshot()` stays the owned copy. `file::write` and the console's unsaved-changes check read through it. `orcvs/tests/allocation.rs` pins that comparing the Cells allocates nothing and that writing a Source File allocates less than one copy of them; the Source File round-trip tests pass unmodified.
