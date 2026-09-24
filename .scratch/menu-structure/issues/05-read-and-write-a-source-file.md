# 05 — Read and write a Source File

**What to build:** The `orcvs` crate turns a Source into Source File text and Source File text into a Source, per `spec.md`'s format and ADR 0054. No UI.

**Blocked by:** 01 — Fix the Grid at 256 by 256.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Writing produces one LF-terminated line per row up to the last row holding content, each with trailing spaces trimmed.
- [ ] Reading accepts LF and CRLF, treats short and missing lines as empty Cells, and places line *n* character *m* at column *m*, row *n*.
- [ ] Reading refuses the whole text, naming line and column, for any byte that is not printable ASCII or space (tab included), a 257th line, or a line past 256 characters. A refusal constructs no Source.
- [ ] The Function reference loader reads through this, and its ragged-text test still holds.
- [ ] A property test: every Source round-trips through write then read unchanged.
- [ ] Boundary tests for 256 lines and 256-character lines accepted, 257 refused, each refused byte class, and CRLF.
- [ ] Scoped gates for `orcvs` and `console` pass.
