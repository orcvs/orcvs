# 08 — Save a Source File

**What to build:** File > Save (⌘S) writes the Source to the open file; File > Save As… (⇧⌘S), or Save with no open file, picks a path with a native dialog first.

**Blocked by:** 05 — Read and write a Source File; 06 — Track the open Source File; 07 — Open a Source File.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Save writes Source File text to the open path and clears the unsaved marker.
- [ ] Save As defaults to the `.orcvs` extension, and its path becomes the open file.
- [ ] A failed write leaves the unsaved marker set and shows the error as a notice.
- [ ] The write does not leave a truncated file on failure: write beside, then rename.
- [ ] Neither item appears on web, and the chords do nothing there.
- [ ] Tests write under an isolated directory and read the result back through `05`.
- [ ] Scoped gates for `console` pass, and `mise run check_wasm` still compiles.
