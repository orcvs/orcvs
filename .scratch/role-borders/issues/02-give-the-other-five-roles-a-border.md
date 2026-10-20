# 02 — Give the other five roles a border

**What to build:** Every Source role can carry a border, not only Function. A Theme document sets `source.<role>.border` and `source.<role>.border.width` for Ordinary, Comment, Number, Note and Bang, and the console draws each one on the same terms `01` set for Function. Each border is a layer above the Grid line and below every overlay border. It applies only when visible and only to occupied Cells, and the winning channel decides the stroke's width.

Bang is painted as itself rather than as a Function, so a Bang Cell takes the Bang border, never the Function border. An operand Cell takes its declared role's border whatever its state: Pending, Valid or Invalid. On an Invalid operand the Diagnostic border then wins over it, as it wins over the Grid line today.

The four built-in Themes set all five transparent at 0.5 point, so nothing a built-in paints changes.

**Blocked by:** 01 — Paint a Function border from the Theme

**Status:** resolved

- [x] The ten properties are in the Theme property catalogue, decoded, range-checked and inherited exactly as `01`'s are.
- [x] All four built-in Themes define all ten explicitly, at transparent and 0.5 point.
- [x] The theming schema's catalogue and the Theme record's values tables list all ten for every built-in, and the record tests pass.
- [x] Paint tests show: each role's visible border strokes its own occupied Cells and no other role's; a Bang Cell takes the Bang border; an Invalid operand shows the Diagnostic border over its role's border; an empty Cell keeps the plain Grid line.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings`, `cargo nextest run --package console --locked` and the `--no-default-features` arm pass.

## Comments

**2026-10-20 — landed.** `source.{ordinary,comment,number,note,bang}.border` and their widths join `01`'s Function pair in the style catalogue and all four built-ins (transparent, 0.5). `style::role_border` maps every Source Paint fact to its role: Unclaimed to Ordinary, an operand to its declared Token in every state, Bang to Bang. Tests: `style::tests::each_role_strokes_its_own_border` (every fact, including Bang and both Invalid operands under the Diagnostic border) and `paint::tests::a_comment_border_frames_its_text_and_not_its_blank_cells` (a blank Cell a Comment claims keeps the Grid line, from written Source). The grayscale shared-grey test reads every role's real border, which `03` will make visible.
