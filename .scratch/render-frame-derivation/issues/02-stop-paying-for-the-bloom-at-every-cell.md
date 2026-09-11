# 02 — Stop paying for the bloom at every Cell

**What to build:** Answer `None` for a Cell outside the Cursor bloom's reach without computing its
distance, breakup and hash first.

**Blocked by:** 01 — Measure what a Render Frame costs before changing anything.

**Status:** needs-triage

- [ ] A Cell outside the bloom radius costs a bounds comparison rather than `signal_breakup` and
      `cell_hash`. The radius is Chebyshev over `DEFAULT_HIGHLIGHT_DOT_SPACING = 7`
      (`orcvs/src/opts.rs:6`), reaching 15 by 15, so at most 225 Cells of the default 1000 can answer
      anything but `None` — and at a resized Grid the proportion only falls.
- [ ] Every existing Render Frame test passes unchanged. A cheaper `None` must be the same `None`:
      `classify_cursor_bloom` (`orcvs/src/render_frame.rs:174-189`) already returns `None` for these
      Cells, so this changes what is paid, never what is answered.
- [ ] The win is stated as a measured difference against issue 01's baseline through
      `source_render_frame`, or the issue says the difference was too small to measure and closes.

## Comments

`needs-triage` on purpose. This is the smallest of the three costs the effort's spec names and it may
well be invisible next to the per-Cell walk and the `Vec` per row that remain either way — which is
exactly what issue 01 exists to find out. Do not promote this before there is a curve to read.

The reason it is written down now rather than later is that it is the one cost in `derive` that is
pure waste rather than work: the Cell is going to answer `None`, the answer does not depend on the
hash, and the hash is computed anyway. The other two costs are paying for something.
