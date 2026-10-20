# 02 — Draw each Grid line once

**What to build:** An empty Grid shows each Grid line once, at `grid.border.width`, on whole physical pixels, with every crossing drawn once. The lines and junctions are non-overlapping pieces of one Mesh placed by ADR 0078's pixel rule. Cells that carry a role, Portal, Locked, Diagnostic or Cursor border keep today's per-Cell border for now, drawn over the lines. See `../spec.md`.

The step takes the Cell as a width and a height snapped per axis, never one square side. If `.scratch/source-cell-size/` ticket 03 has landed it reads both from the viewport; if not, both are today's snapped side.

**Blocked by:** 01 — Measure the shapes step.

**Status:** ready-for-agent

- [ ] Built test-first at the shapes step: on an empty Grid every edge is one piece, pieces never overlap, every crossing has a junction piece, and every piece is whole-pixel, at 1×, 2× and a fractional scale, with equal and unequal Cell width and height.
- [ ] A line under one physical pixel covers one pixel at its alpha scaled by its width.
- [ ] A one-pixel junction is present in the Shapes (epaint's rectangle path would round it away; the Mesh is built from quads).
- [ ] Unclaimed Cells draw no per-Cell border any more; the Paint's Grid line answer is the lattice's.
- [ ] Tests that pinned a Grid line as a per-Cell rectangle are rewritten to the lattice, not deleted.
- [ ] The running console at 1× and 2× is compared with a screenshot from before: plain Grid lines are half their previous visible width and crisp.
- [ ] The benchmark from 01 is run by CI on the pull request.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings`, `cargo nextest run --package console --locked` and the `--no-default-features` arm pass; `mise run check_wasm` passes.
