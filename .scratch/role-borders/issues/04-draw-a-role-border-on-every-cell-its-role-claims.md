# 04 — Draw a role border on every Cell its role claims

**What to build:** A role border frames the whole span its role claims, not only the Cells that hold content. A Comment's blank Cells between its `||` marks take the Comment border, as they already take the Comment fill, so a framed Comment reads as one span rather than framed glyphs over Grid lines. A Pending operand slot takes its declared Token's border the same way.

The rule `01` set was occupancy: a role border drew only on a Cell holding content. It becomes the claim:

- A Cell a Function, Bang, Comment, Number or Note claims draws that role's border, whether it holds content or not.
- An unclaimed Cell draws as Ordinary. It draws the Ordinary border only while it holds content, so an empty Cell keeps the plain Grid line and the Ordinary border never frames the whole Grid.

The visible-only rule, the layer order and the width rule are unchanged.

**Blocked by:** None — can start immediately. Land it after `cursor-foreground/01` and `02`, which edit the same paint code.

**Status:** resolved

- [x] A blank Cell a Comment claims draws the Comment border, from written Source.
- [x] A Pending operand slot draws its declared Token's border.
- [x] An empty unclaimed Cell keeps the plain Grid line and `grid.border.width`, even with a visible Ordinary border; an unclaimed Cell holding content draws the Ordinary border.
- [x] The existing tests that pinned occupancy are rewritten to the claim rule rather than deleted: the Comment wiring test and the empty-Cell test.
- [x] The theming schema's composition step and the console's Theme record state the claim rule.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings`, `cargo nextest run --package console --locked` and the `--no-default-features` arm pass.
