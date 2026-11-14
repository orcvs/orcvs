//!
//! Info: what the Cursor's Cell belongs to, read from the Render Frame and the
//! Function table, as the entries the Panel draws in two lines.
//!

use orcvs::grid::Position;
use orcvs::render_frame::RenderFrame;
use orcvs::source::Token;

///
/// One entry of Info: the Grid Cells line one paints, as the Source paints
/// them, and the label line two shows beneath them.
///
#[derive(Debug, PartialEq)]
pub(crate) struct Entry {
    /// The Cells the entry shows, empty for a summary with no Cells of its
    /// own.
    pub(crate) cells: Vec<Position>,
    /// What the Cells are: the Function's or Comment's name, a Role, or what
    /// the answer is.
    pub(crate) label: String,
    /// Whether the label is drawn in the strong text colour (a name) or the
    /// weak one (a Role or a summary).
    pub(crate) strong: bool,
}

/// The prefix a summary of a Function's answer starts with, which the label
/// beneath that answer drops.
const ANSWER_PREFIX: &str = "answers ";

///
/// What the Cursor's Cell in `frame` belongs to. On any Cell of a Function's
/// Span: its spelling over its name, each Role's Cells over the Role's name (a
/// Position's two Numbers are one Role), then, where the answer has a fixed
/// place, its Output Portal's Cells over what it answers, or else the summary
/// alone. A Comment's opening pair is named. Any other Cell, empty or not,
/// answers no entries: a standalone pair of Cells is diagnosed text, not an
/// Operand Literal.
///
pub(crate) fn info(frame: &RenderFrame) -> Vec<Entry> {
    let Some(owner) = frame.cursor_function() else {
        let grid = frame.grid();
        return frame
            .at(frame.cursor())
            .claim()
            .filter(|claim| claim.token == Token::Comment)
            .map(|claim| {
                let cells = claim
                    .cells
                    .clone()
                    .take(2)
                    .filter_map(|index| grid.cell_index(index))
                    .map(|index| grid.position_at(index))
                    .collect();
                vec![Entry {
                    cells,
                    label: "Comment".to_owned(),
                    strong: true,
                }]
            })
            .unwrap_or_default();
    };
    let function = owner.function;
    let mut entries = vec![Entry {
        cells: owner.anchor.positions().collect(),
        label: function.name().to_owned(),
        strong: true,
    }];
    let mut operands = owner.operands.iter().copied();
    for role in function.roles() {
        entries.push(Entry {
            cells: operands
                .by_ref()
                .take(role.operands)
                .flat_map(|span| span.positions())
                .collect(),
            label: role.name.to_owned(),
            strong: false,
        });
    }
    let summary = function.summary();
    entries.push(match owner.answer {
        Some(span) => Entry {
            cells: span.positions().collect(),
            label: summary.trim_prefix(ANSWER_PREFIX).to_owned(),
            strong: false,
        },
        None => Entry {
            cells: Vec::new(),
            label: summary.to_owned(),
            strong: false,
        },
    });
    entries
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

    use super::{info, write_rows};

    ///
    /// Info with `rows` written from the top-left of a 16 by 4 Grid and the
    /// Cursor at `cursor`, each entry read as `[Cells]label`, an empty Cell
    /// as `.`.
    ///
    fn line(rows: &[&str], cursor: (usize, usize)) -> String {
        let mut orcvs = Orcvs::with_shape(16, 4).expect("the test runtime");
        write_rows(&orcvs, rows);
        let grid = orcvs.grid();
        orcvs.select(grid.position(cursor.0, cursor.1).expect("inside the Grid"));
        let frame = orcvs.render_frame();
        info(&frame)
            .into_iter()
            .map(|entry| {
                let cells: String = entry
                    .cells
                    .iter()
                    .map(|&position| frame.at(position).content().unwrap_or('.'))
                    .collect();
                format!("[{cells}]{}", entry.label)
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[tokio::test]
    async fn a_functions_anchor_names_it_its_operands_and_its_answer() {
        assert_eq!(
            line(&[".+0304", "07"], (0, 0)),
            "[.+]Add [03]left [04]right [07]left + right"
        );
    }

    #[tokio::test]
    async fn an_operand_cell_describes_the_function_that_owns_it() {
        assert_eq!(
            line(&[".+0304"], (5, 0)),
            "[.+]Add [03]left [04]right [..]left + right"
        );
        assert_eq!(
            line(&[".+.x020304"], (6, 0)),
            "[.x]Multiply [02]left [03]right [..]left * right",
            "a nested Function owns its own operands"
        );
    }

    #[tokio::test]
    async fn a_function_cut_short_by_the_row_edge_names_the_cells_it_has() {
        assert_eq!(
            line(&["", "", "", "             .+0"], (13, 3)),
            "[.+]Add [0]left []right []answers left + right",
            "the last row has no row beneath for an answer, and its operands stop at the edge"
        );
    }

    #[tokio::test]
    async fn an_effect_function_ends_on_its_summary() {
        assert_eq!(
            line(&["!>0045C4"], (0, 0)),
            "[!>]Raw Play [00]channel [45]velocity [C4]note []sends a Note On, or a Note Off at velocity 00"
        );
    }

    #[tokio::test]
    async fn a_position_reads_as_one_operand() {
        assert_eq!(
            line(&["=$00020004"], (0, 0)),
            "[=$]Absolute Copy [0002]from [0004]to []copies the pair at from onto the pair at to"
        );
        assert_eq!(
            line(&["&$0A03"], (2, 0)),
            "[&$]Absolute Read [0A03]at [..]reads the pair at a Position"
        );
        assert_eq!(
            line(&["@$0A0307"], (0, 0)),
            "[@$]Absolute Write [0A03]at [07]value []writes value at a Position"
        );
    }

    #[tokio::test]
    async fn a_comment_and_a_bang_are_named_by_their_language_unit() {
        assert_eq!(line(&["||note||"], (3, 0)), "[||]Comment");
        assert_eq!(
            line(&["**"], (1, 0)),
            "[**]Bang []activates the roots aligned with it"
        );
    }

    #[tokio::test]
    async fn an_empty_cell_or_a_standalone_pair_leaves_info_blank() {
        assert_eq!(line(&[".+0304"], (0, 3)), "");
        assert_eq!(
            line(&["07"], (0, 0)),
            "",
            "diagnosed text, not a Language Unit"
        );
    }
}
