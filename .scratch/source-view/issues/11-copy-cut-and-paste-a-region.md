# 11: Copy, cut and paste a Region

**What to build:** Copy puts the Region's rows on the system clipboard as text joined by newlines, trailing spaces kept. Cut copies, then empties the Region. Paste writes from the Region's top-left, spaces included, clipped at the Grid's edges, and the Region becomes the rectangle that landed. A character that cannot be a Cell lands as an empty Cell, and `\r\n` is one row break. Works on native and WASM. See ADR 0046.

**Blocked by:** 10

**Status:** ready-for-agent

- [ ] Copy puts the Region's rows on the clipboard, joined by newlines, with trailing spaces kept.
- [ ] Cut copies and then empties every Cell of the Region.
- [ ] Paste writes from the Region's top-left, spaces included, clipped at the Grid's edges, and the Region becomes what landed.
- [ ] A pasted character that cannot be a Cell lands as an empty Cell, and `\r\n` is one row break.
- [ ] The clipboard chords never write their own characters to the Source.
