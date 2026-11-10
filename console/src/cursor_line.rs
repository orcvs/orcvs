//!
//! The Cursor Line's text: what the Cursor's Cell belongs to, read from the
//! Render Frame and the Function table. The Panel draws it.
//!

use orcvs::render_frame::RenderFrame;
use orcvs::source::{Span, Token};

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
/// The Cursor Line's text: the `lead` from the left edge, `answer` held at the
/// right edge. Either is empty when it has nothing to say.
///
#[derive(Debug, Default, PartialEq)]
pub(crate) struct CursorLine {
    pub(crate) lead: Vec<Part>,
    pub(crate) answer: Vec<Part>,
}

/// The characters a Function's spelling and name fill before its operands, so
/// operands start in one column for most Functions.
const IDENTITY_WIDTH: usize = 18;
/// The space between the identity, the operands and the summary.
const SECTION_GAP: usize = 3;
/// The space between two operands.
const OPERAND_GAP: usize = 2;

fn part(text: impl Into<String>, strong: bool) -> Part {
    Part {
        text: text.into(),
        strong,
    }
}

///
/// What the Cursor's Cell in `frame` belongs to. On any Cell of a Function's
/// Span: its spelling and name, each Role's name and Cells, a Position's two
/// Numbers read as one Role, then its summary. A Function whose answer has a
/// fixed place answers with its Output Portal's two Cells and that Portal's
/// Position. An empty Cell reads `.`. A Comment is named. Any other Cell
/// answers no text: a standalone pair of Cells is diagnosed text, not an
/// Operand Literal.
///
pub(crate) fn cursor_line(frame: &RenderFrame) -> CursorLine {
    let mut line = CursorLine::default();
    let Some(owner) = frame.cursor_function() else {
        if frame
            .at(frame.cursor())
            .claim()
            .is_some_and(|claim| claim.token == Token::Comment)
        {
            line.lead.push(part("|| Comment", true));
        }
        return line;
    };
    let function = owner.function;
    // The Cells' text, reading an empty Cell as `.`.
    let pair = |span: Span| -> String {
        span.positions()
            .map(|position| frame.at(position).content().unwrap_or('.'))
            .collect()
    };
    let identity = format!("{} {}", function.cells(), function.name());
    let pad = IDENTITY_WIDTH.saturating_sub(identity.len()) + SECTION_GAP;
    line.lead.push(part(identity, true));
    line.lead.push(part(" ".repeat(pad), false));
    let mut operands = owner.operands.iter().copied();
    for (index, role) in function.roles().iter().enumerate() {
        if index > 0 {
            line.lead.push(part(" ".repeat(OPERAND_GAP), false));
        }
        let value = operands
            .by_ref()
            .take(role.operands)
            .map(pair)
            .collect::<Vec<_>>()
            .join(" ");
        line.lead.push(part(format!("{} ", role.name), false));
        line.lead.push(part(value, true));
    }
    if !function.roles().is_empty() {
        line.lead.push(part(" ".repeat(SECTION_GAP), false));
    }
    line.lead.push(part(function.summary(), false));
    if let Some(span) = owner.answer {
        let portal = span.first_position();
        line.answer.push(part("\u{2192} ", false));
        line.answer.push(part(pair(span), true));
        line.answer.push(part(
            format!(" at {:02X} {:02X}", portal.x(), portal.y()),
            false,
        ));
    }
    line
}

///
/// Writes `rows` from the top-left of `orcvs`'s Grid, leaving a space's Cell
/// empty.
///
#[cfg(test)]
pub(crate) fn write_rows(orcvs: &orcvs::app::Orcvs, rows: &[&str]) {
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
}

#[cfg(test)]
mod tests {
    use orcvs::app::Orcvs;

    use super::{Part, cursor_line, write_rows};

    ///
    /// The Cursor Line with `rows` written from the top-left of a 16 by 4
    /// Grid and the Cursor at `cursor`.
    ///
    fn line(rows: &[&str], cursor: (usize, usize)) -> String {
        let mut orcvs = Orcvs::with_shape(16, 4).expect("the test runtime");
        write_rows(&orcvs, rows);
        let grid = orcvs.grid();
        orcvs.select(grid.position(cursor.0, cursor.1).expect("inside the Grid"));
        let text =
            |parts: Vec<Part>| -> String { parts.into_iter().map(|part| part.text).collect() };
        let line = cursor_line(&orcvs.render_frame());
        let (lead, answer) = (text(line.lead), text(line.answer));
        if answer.is_empty() {
            lead
        } else {
            format!("{lead} | {answer}")
        }
    }

    #[tokio::test]
    async fn a_functions_anchor_names_it_its_operands_and_its_answer() {
        assert_eq!(
            line(&[".+0304", "07"], (0, 0)),
            ".+ Add               left 03  right 04   answers left + right | → 07 at 00 01"
        );
    }

    #[tokio::test]
    async fn an_operand_cell_describes_the_function_that_owns_it() {
        assert_eq!(
            line(&[".+0304"], (5, 0)),
            ".+ Add               left 03  right 04   answers left + right | → .. at 00 01"
        );
        assert_eq!(
            line(&[".+.x020304"], (6, 0)),
            ".x Multiply          left 02  right 03   answers left * right | → .. at 02 01",
            "a nested Function owns its own operands"
        );
    }

    #[tokio::test]
    async fn a_function_cut_short_by_the_row_edge_names_the_cells_it_has() {
        assert_eq!(
            line(&["", "", "", "             .+0"], (13, 3)),
            ".+ Add               left 0  right    answers left + right",
            "the last row has no row beneath for an answer, and its operands stop at the edge"
        );
    }

    #[tokio::test]
    async fn an_effect_function_ends_on_its_summary() {
        assert_eq!(
            line(&["!>0045C4"], (0, 0)),
            "!> Raw Play          channel 00  velocity 45  note C4   sends a Note On, or a Note Off at velocity 00"
        );
    }

    #[tokio::test]
    async fn a_position_reads_as_one_operand() {
        assert_eq!(
            line(&["=$00020004"], (0, 0)),
            "=$ Absolute Copy     from 00 02  to 00 04   copies the pair at from onto the pair at to"
        );
        assert_eq!(
            line(&["&$0A03"], (2, 0)),
            "&$ Absolute Read     at 0A 03   reads the pair at a Position | → .. at 00 01"
        );
        assert_eq!(
            line(&["@$0A0307"], (0, 0)),
            "@$ Absolute Write    at 0A 03  value 07   writes value at a Position"
        );
    }

    #[tokio::test]
    async fn a_comment_and_a_bang_are_named_by_their_language_unit() {
        assert_eq!(line(&["||note||"], (3, 0)), "|| Comment");
        assert_eq!(
            line(&["**"], (1, 0)),
            "** Bang              activates the roots aligned with it"
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
