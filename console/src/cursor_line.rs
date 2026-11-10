//!
//! The Cursor Line's text: what the Cursor's Cell belongs to, read from the
//! Render Frame and the Function table. The Panel draws it.
//!

use orcvs::render_frame::RenderFrame;
use orcvs::source::Token;

///
/// One run of the Cursor Line's text, and whether it is drawn in the strong
/// text colour (a name or a value) or the weak one (a summary, a role or a
/// Position).
///
#[derive(Debug, PartialEq)]
pub(crate) struct Part {
    pub(crate) text: String,
    pub(crate) strong: bool,
}

///
/// What the Cursor's Cell in `frame` belongs to. On any Cell of a Function's
/// Span: its name, spelling and summary, each operand's role and two Cells,
/// then, for a Function whose answer has a fixed place, `answer`, its Output
/// Portal's two Cells and that Portal's Position. An empty Cell reads `.`.
/// A Comment is named. Any other Cell answers no text: a standalone pair of
/// Cells is diagnosed text, not an Operand Literal.
///
pub(crate) fn cursor_line(frame: &RenderFrame) -> Vec<Part> {
    let mut parts = Vec::new();
    let mut push = |gap: &str, text: String, strong: bool| {
        if !parts.is_empty() {
            parts.push(Part {
                text: gap.to_owned(),
                strong: false,
            });
        }
        parts.push(Part { text, strong });
    };
    let Some(owner) = frame.cursor_function() else {
        if frame
            .at(frame.cursor())
            .claim()
            .is_some_and(|claim| claim.token == Token::Comment)
        {
            push("", "Comment".to_owned(), true);
        }
        return parts;
    };
    let function = owner.function;
    // The Cells' text, reading an empty Cell as `.`. Each range is one the
    // Language Map or the Portal reservation drew inside the Grid.
    let pair = |cells: std::ops::Range<usize>| -> String {
        frame
            .cells()
            .get(cells)
            .unwrap_or_default()
            .iter()
            .map(|cell| cell.content().unwrap_or('.'))
            .collect()
    };
    push("", function.name().to_owned(), true);
    push("  ", function.cells().to_string(), true);
    push("  ", function.summary().to_owned(), false);
    for (role, cells) in function.roles().iter().zip(&owner.operands) {
        push("   ", (*role).to_owned(), false);
        push(" ", pair(cells.clone()), true);
    }
    if let Some(cells) = &owner.answer {
        push("   ", "answer".to_owned(), false);
        push(" ", pair(cells.clone()), true);
        let grid = frame.grid();
        let portal = grid.position_at(
            grid.cell_index(cells.start)
                .expect("the Portal reservation lies inside its Grid"),
        );
        push(
            "  ",
            format!("at {:02X} {:02X}", portal.x(), portal.y()),
            false,
        );
    }
    parts
}

#[cfg(test)]
mod tests {
    use orcvs::app::Orcvs;

    use super::cursor_line;

    ///
    /// The Cursor Line with `rows` written from the top-left of a 16 by 4
    /// Grid and the Cursor at `cursor`.
    ///
    fn line(rows: &[&str], cursor: (usize, usize)) -> String {
        let mut orcvs = Orcvs::with_shape(16, 4).expect("the test runtime");
        let grid = orcvs.grid();
        for (y, row) in rows.iter().enumerate() {
            for (x, content) in row.chars().enumerate().filter(|(_, c)| *c != ' ') {
                let cell = grid.index(grid.position(x, y).expect("inside the Grid"));
                orcvs
                    .source()
                    .set(cell, &content.to_string())
                    .expect("a Cell");
            }
        }
        orcvs.select(grid.position(cursor.0, cursor.1).expect("inside the Grid"));
        cursor_line(&orcvs.render_frame())
            .into_iter()
            .map(|part| part.text)
            .collect()
    }

    #[tokio::test]
    async fn a_functions_anchor_names_it_its_operands_and_its_answer() {
        assert_eq!(
            line(&[".+0304", "07"], (0, 0)),
            "Add  .+  answers left + right   left 03   right 04   answer 07  at 00 01"
        );
    }

    #[tokio::test]
    async fn an_operand_cell_describes_the_function_that_owns_it() {
        assert_eq!(
            line(&[".+0304"], (5, 0)),
            "Add  .+  answers left + right   left 03   right 04   answer ..  at 00 01"
        );
        assert_eq!(
            line(&[".+.x020304"], (6, 0)),
            "Multiply  .x  answers left * right   left 02   right 03   answer ..  at 02 01",
            "a nested Function owns its own operands"
        );
    }

    #[tokio::test]
    async fn a_function_cut_short_by_the_row_edge_names_the_cells_it_has() {
        assert_eq!(
            line(&["", "", "", "             .+0"], (13, 3)),
            "Add  .+  answers left + right   left 0   right ",
            "the last row has no row beneath for an answer, and its operands stop at the edge"
        );
    }

    #[tokio::test]
    async fn an_effect_function_ends_after_its_operands() {
        assert_eq!(
            line(&["!>0045C4"], (0, 0)),
            "Play  !>  sends a Note On   channel 00   velocity 45   note C4"
        );
    }

    #[tokio::test]
    async fn a_comment_and_a_bang_are_named_by_their_language_unit() {
        assert_eq!(line(&["||note||"], (3, 0)), "Comment");
        assert_eq!(
            line(&["**"], (1, 0)),
            "Bang  **  activates the roots aligned with it"
        );
    }

    #[tokio::test]
    async fn an_empty_cell_or_a_standalone_pair_leaves_the_line_blank() {
        assert_eq!(line(&[".+0304"], (0, 3)), "");
        assert_eq!(
            line(&["07"], (0, 0)),
            "",
            "diagnosed text, not a Language Unit"
        );
    }
}
