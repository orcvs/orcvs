use std::sync::Arc;

use crate::{
    grid::{Grid, Position},
    opts::{CursorBloomRadius, SectorSeamSpacing},
    region::Region,
    source::{Claim, Diagnostic, OperandState, SourcePaint, SourceRevision, Span, Token},
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct RenderFrameConfig {
    pub sector_seam_spacing: SectorSeamSpacing,
    pub cursor_bloom_radius: CursorBloomRadius,
}

#[derive(Clone, Debug)]
pub struct RenderCell {
    position: Position,
    content: Option<char>,
    claim: Option<Arc<Claim>>,
    output_portal: bool,
}

impl RenderCell {
    pub fn position(&self) -> Position {
        self.position
    }

    pub fn content(&self) -> Option<char> {
        self.content
    }

    ///
    /// The parser's claim on this Cell, when one claims it.
    ///
    /// `None` means no positioned entry covers this Cell: an empty unclaimed
    /// Cell, or leftover content no Expression claimed. Each claim is stored
    /// once and shared by every Cell it covers.
    ///
    pub fn claim(&self) -> Option<&Claim> {
        self.claim.as_deref()
    }

    /// The language fact this Cell's Source Paint presents.
    ///
    /// Whether an operand slot is written is answered once, as its claim is
    /// built, and read here from the claim every covered Cell shares
    /// ([`Claim::written`]). Paint therefore spends no lookup or claim-range
    /// walk per drawn Cell.
    pub fn source_paint(&self) -> SourcePaint {
        let Some(claim) = self.claim() else {
            return SourcePaint::Unclaimed;
        };
        match claim.token {
            Token::Bang => SourcePaint::Bang,
            Token::Comment => SourcePaint::Comment,
            Token::Function if claim.atom.is_some() => SourcePaint::Function,
            Token::Function => SourcePaint::Unclaimed,
            Token::Char => unreachable!("the Parser never creates a positioned Char claim"),
            token @ (Token::Number | Token::Note | Token::Atom | Token::Sequence) => {
                SourcePaint::Operand {
                    token,
                    state: if claim.atom.is_some() {
                        OperandState::Valid
                    } else if claim.written {
                        OperandState::Invalid
                    } else {
                        OperandState::Pending
                    },
                }
            }
        }
    }

    ///
    /// Whether this Cell draws as a root Function's Output Portal, derived
    /// from the current Source revision alone and known before any Tick runs
    /// (`.scratch/syntax-highlighting/issues/05`'s Answer).
    ///
    /// `true` covers the Cell pair from the Output Portal for a Function that
    /// can only answer a scalar. A Function that can answer a Sequence covers
    /// the fitted highlight `SourceRevision::output_portal_highlight` derives
    /// (`.scratch/syntax-highlighting/issues/12`): at least four Cells from
    /// the Output Portal, then each following written Cell pair, clipped to
    /// the Reservation — not the whole Reservation, which is what the Tick
    /// scheduler still reserves. A nested Function, a Terminal Output
    /// Function, Halt, and a Source-writing Function (including an Advance's
    /// cleared anchor) never set it, and neither does a scalar destination the
    /// row edge leaves no room for.
    ///
    pub fn output_portal(&self) -> bool {
        self.output_portal
    }
}

///
/// One Expression this Render Frame was derived from.
///
/// Carried once per Expression rather than copied onto every Cell the Span
/// covers. The console resolves a Cell to its Expression; a diagnostic and
/// an executability decision belong to the Expression.
///
#[derive(Clone, Debug)]
pub struct RenderExpression {
    span: Span,
    diagnostic: Option<Diagnostic>,
    root: Option<Position>,
}

impl RenderExpression {
    pub fn span(&self) -> Span {
        self.span
    }

    pub fn diagnostic(&self) -> Option<&Diagnostic> {
        self.diagnostic.as_ref()
    }

    /// The first Function anchor when this is a complete executable Expression.
    pub fn root(&self) -> Option<Position> {
        self.root
    }
}

#[derive(Clone, Debug)]
pub struct RenderFrame {
    grid: Grid,
    region: Region,
    cursor_visible: bool,
    sector_seam_spacing: SectorSeamSpacing,
    cursor_bloom_radius: CursorBloomRadius,
    cells: Vec<RenderCell>,
    expressions: Vec<RenderExpression>,
    lexical_diagnostics: Vec<Diagnostic>,
}

impl RenderFrame {
    pub(crate) fn derive(
        source: SourceRevision,
        region: Region,
        cursor_visible: bool,
        config: RenderFrameConfig,
    ) -> Self {
        let grid = source.grid();
        grid.assert_owns(region.anchor());
        grid.assert_owns(region.cursor());
        let claims = source.claims_by_cell();
        let output_portals = source.output_portal_highlight();
        let cells = grid
            .positions_by_row()
            .flatten()
            .map(|position| {
                let index = grid.index(position).get();
                RenderCell {
                    position,
                    content: source.content_at(position),
                    claim: claims[index].clone(),
                    output_portal: output_portals[index],
                }
            })
            .collect();
        let expressions = source
            .language_map()
            .expressions()
            .map(|expression| RenderExpression {
                span: expression.span(),
                diagnostic: expression.diagnostic().cloned(),
                root: expression.root(),
            })
            .collect();
        let lexical_diagnostics = source
            .language_map()
            .lexical_diagnostics()
            .cloned()
            .collect();
        Self {
            grid,
            region,
            cursor_visible,
            sector_seam_spacing: config.sector_seam_spacing,
            cursor_bloom_radius: config.cursor_bloom_radius,
            cells,
            expressions,
            lexical_diagnostics,
        }
    }

    ///
    /// The Grid this Render Frame was derived from.
    ///
    /// Carried rather than recovered. The derivation already holds it to assert
    /// the selected Position belongs to it, and `Grid` is `Copy`, so keeping
    /// the fact costs nothing.
    ///
    pub fn grid(&self) -> Grid {
        self.grid
    }

    ///
    /// The Cursor: the Position the derivation was given and asserted
    /// the Grid owns.
    ///
    /// Carried rather than recovered. The derivation already holds the Position, and
    /// `Position` is `Copy`, so keeping it costs nothing. It spares every
    /// consumer scanning the Cells for the one whose `selected` flag is set —
    /// a search whose answer the type of `&[RenderCell]` cannot state.
    ///
    pub fn cursor(&self) -> Position {
        self.region.cursor()
    }

    ///
    /// The Region: the anchor and the Cursor the derivation was given.
    ///
    /// Carried beside the Cursor, which is its live end, so a console can tint
    /// the Region without being told which Cell the Cursor is twice. It is
    /// running state rather than Source, so it is never stored with one.
    ///
    pub fn region(&self) -> Region {
        self.region
    }

    ///
    /// Whether the Cursor is visible in this Frame.
    ///
    /// Carried from derivation rather than recovered from a Cell
    /// flag. Visibility is a fact about the Frame, not about any one Cell.
    ///
    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    ///
    /// The sector-seam period, in Cells, this Frame was derived with.
    ///
    /// Presentation configuration handed across so the console can draw the
    /// seams; the period itself still lives on `Opts`.
    ///
    pub fn sector_seam_spacing(&self) -> SectorSeamSpacing {
        self.sector_seam_spacing
    }

    ///
    /// The Cursor Bloom radius, in Cells, this Frame was derived with.
    ///
    /// Presentation configuration handed across so the console can grade the
    /// bloom; the radius itself still lives on `Opts`.
    ///
    pub fn cursor_bloom_radius(&self) -> CursorBloomRadius {
        self.cursor_bloom_radius
    }

    ///
    /// The Cell at `position`.
    ///
    /// Indexed through [`Grid::index`] in row-major order over the flat Cells.
    ///
    pub fn at(&self, position: Position) -> &RenderCell {
        self.grid.assert_owns(position);
        &self.cells[self.grid.index(position).get()]
    }

    ///
    /// Every Cell of the Grid in row-major order.
    ///
    pub fn cells(&self) -> &[RenderCell] {
        &self.cells
    }

    ///
    /// Every Expression this Frame was derived from, in Source order.
    ///
    pub fn expressions(&self) -> &[RenderExpression] {
        &self.expressions
    }

    ///
    /// The Expression whose Span covers `position`, when one does.
    ///
    /// Expression Spans are disjoint, so at most one Expression answers for a
    /// Cell, matching [`crate::source::LanguageMap::token_at`]. The Language
    /// Map property
    /// `expression_spans_are_disjoint_and_name_cells_the_grid_can_answer_for`
    /// guards that, along with each positioned entry lying inside its own
    /// Expression's Span.
    ///
    pub fn expression_at(&self, position: Position) -> Option<&RenderExpression> {
        self.grid.assert_owns(position);
        self.expressions.iter().rev().find(|expression| {
            expression
                .span
                .positions()
                .any(|covered| covered == position)
        })
    }

    ///
    /// Unmatched-character diagnostics this revision established.
    ///
    /// Copied from the Language Map. A Cell can sit inside one of these
    /// without sitting inside an Expression.
    ///
    pub fn lexical_diagnostics(&self) -> &[Diagnostic] {
        &self.lexical_diagnostics
    }

    ///
    /// Whether an Expression diagnostic or a lexical diagnostic covers
    /// `position`.
    ///
    pub fn diagnostic_covers(&self, position: Position) -> bool {
        self.grid.assert_owns(position);
        let expression_diagnostic = self
            .expression_at(position)
            .and_then(RenderExpression::diagnostic)
            .is_some_and(|diagnostic| diagnostic_span_covers(diagnostic, position));
        let lexical = self
            .lexical_diagnostics
            .iter()
            .any(|diagnostic| diagnostic_span_covers(diagnostic, position));
        expression_diagnostic || lexical
    }
}

fn diagnostic_span_covers(diagnostic: &Diagnostic, position: Position) -> bool {
    diagnostic
        .span()
        .positions()
        .any(|covered| covered == position)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Barrier};

    use crate::{
        grid::{CellIndex, Grid},
        opts::{CursorBloomRadius, SectorSeamSpacing},
        region::Region,
        render_frame::{RenderFrame, RenderFrameConfig},
        source::{OperandState, SourceCommander, SourcePaint, Tick, Token},
    };

    ///
    /// The index `grid` mints for `idx`. A Cell is named by an index its Grid
    /// minted, so a test states the number and the Grid answers with the Cell.
    ///
    fn cell(grid: Grid, idx: usize) -> CellIndex {
        grid.cell_index(idx).expect("inside the Grid")
    }

    fn cell_at(frame: &RenderFrame, position: crate::grid::Position) -> &super::RenderCell {
        frame.at(position)
    }

    ///
    /// The Token the claim on `position` declares, or `None` when nothing
    /// claims it. `RenderCell::token()` answered this directly before
    /// `syntax-highlighting/09` removed it in favour of `claim()`; every test
    /// below that named a Token by position now reads it from the claim.
    ///
    fn token_at(frame: &RenderFrame, position: crate::grid::Position) -> Option<Token> {
        frame.at(position).claim().map(|claim| claim.token)
    }

    #[test]
    fn render_frame_is_a_complete_row_structured_visual_snapshot() {
        let grid = Grid::new(2, 2);
        let source = SourceCommander::new(grid);
        source.set(cell(grid, 1), "x").unwrap();
        let selected = grid.position(1, 0).unwrap();

        let frame = RenderFrame::derive(
            source.read_revision(),
            Region::at(grid, selected),
            true,
            RenderFrameConfig {
                sector_seam_spacing: SectorSeamSpacing::new(2).unwrap(),
                cursor_bloom_radius: CursorBloomRadius::new(1).unwrap(),
            },
        );

        assert_eq!(frame.grid().rows(), 2);
        assert_eq!(frame.grid().columns(), 2);
        assert_eq!(frame.cursor(), selected);
        assert!(frame.cursor_visible());
        assert_eq!(
            frame.at(grid.position(0, 0).unwrap()).position(),
            grid.position(0, 0).unwrap()
        );
        assert_eq!(token_at(&frame, grid.position(0, 0).unwrap()), None);
        assert_eq!(frame.at(grid.position(1, 0).unwrap()).content(), Some('x'));
        // A character standing where a Function goes is classified there,
        // whether or not the table holds its spelling.
        assert_eq!(
            token_at(&frame, grid.position(1, 0).unwrap()),
            Some(Token::Function)
        );
        assert_eq!(token_at(&frame, grid.position(0, 1).unwrap()), None);
    }

    #[test]
    fn only_complete_bang_units_receive_bang_glyphs() {
        let grid = Grid::new(4, 1);
        let source = SourceCommander::new(grid);
        for (index, content) in "***x".chars().enumerate() {
            source.set(cell(grid, index), &content.to_string()).unwrap();
        }

        let frame = RenderFrame::derive(
            source.read_revision(),
            Region::at(grid, grid.origin()),
            false,
            RenderFrameConfig {
                sector_seam_spacing: SectorSeamSpacing::new(2).unwrap(),
                cursor_bloom_radius: CursorBloomRadius::new(1).unwrap(),
            },
        );

        assert_eq!(
            token_at(&frame, grid.position(0, 0).unwrap()),
            Some(Token::Bang)
        );
        assert_eq!(
            token_at(&frame, grid.position(1, 0).unwrap()),
            Some(Token::Bang)
        );
        // The third `*` is not half a Bang. It opens an Expression of its own
        // whose spelling `*x` the Function table does not hold, and the `x`
        // opens the one after that — each classified where a Function goes,
        // because that is where each of them stands.
        assert_eq!(
            token_at(&frame, grid.position(2, 0).unwrap()),
            Some(Token::Function)
        );
        assert_eq!(
            token_at(&frame, grid.position(3, 0).unwrap()),
            Some(Token::Function)
        );
    }

    #[test]
    fn a_self_banging_function_is_painted_as_a_function() {
        let grid = Grid::new(2, 1);
        let source = SourceCommander::new(grid);
        source.set(cell(grid, 0), ">").unwrap();
        source.set(cell(grid, 1), ">").unwrap();

        let frame = RenderFrame::derive(
            source.read_revision(),
            Region::at(grid, grid.origin()),
            false,
            RenderFrameConfig {
                sector_seam_spacing: SectorSeamSpacing::new(2).unwrap(),
                cursor_bloom_radius: CursorBloomRadius::new(1).unwrap(),
            },
        );

        // `>>` painted as an ordinary character while it was its own Atom
        // variant, which mapped to leftover Char. It is a row of the Function
        // table now, so it is painted where every other Function is. The change
        // is visible and it is a correction: these two Cells spell a Function.
        assert_eq!(
            token_at(&frame, grid.position(0, 0).unwrap()),
            Some(Token::Function)
        );
        assert_eq!(
            token_at(&frame, grid.position(1, 0).unwrap()),
            Some(Token::Function)
        );
    }

    #[test]
    fn a_valid_function_with_unbound_operands_shares_each_parsers_claim() {
        // `.+c40G`: Addition's two Number operands. `c4` is lowercase and `0G`
        // is not hexadecimal, so both fail `Token::Number::decode` and the
        // Parser records each as `(Token::Number, None)`. The Function itself
        // bound. Each claim is stored once and shared by every Cell it covers.
        let grid = Grid::new(6, 1);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, ".+c40G");
        let frame = derive_frame(&source, grid.origin());

        let function = frame.at(grid.position(0, 0).unwrap()).claim().expect(".+");
        assert_eq!(function.cells, 0..2);
        assert_eq!(function.token, Token::Function);
        assert_eq!(
            function.atom,
            Some(lang::Atom::Function(lang::Function::Add))
        );
        assert!(std::ptr::eq(
            function,
            frame.at(grid.position(1, 0).unwrap()).claim().expect(".+")
        ));

        let first = frame.at(grid.position(2, 0).unwrap()).claim().expect("c4");
        assert_eq!(first.cells, 2..4);
        assert_eq!(first.token, Token::Number);
        assert_eq!(first.atom, None);
        assert!(std::ptr::eq(
            first,
            frame.at(grid.position(3, 0).unwrap()).claim().expect("c4")
        ));
        assert!((2..4).all(|column| {
            frame.at(grid.position(column, 0).unwrap()).source_paint()
                == SourcePaint::Operand {
                    token: Token::Number,
                    state: OperandState::Invalid,
                }
        }));

        let second = frame.at(grid.position(4, 0).unwrap()).claim().expect("0G");
        assert_eq!(second.cells, 4..6);
        assert_eq!(second.token, Token::Number);
        assert_eq!(second.atom, None);
        assert!(std::ptr::eq(
            second,
            frame.at(grid.position(5, 0).unwrap()).claim().expect("0G")
        ));
    }

    #[test]
    fn a_bound_number_sits_beside_an_unbound_blank_operand_claim() {
        // `.+01  `: the first Number binds; the second operand is two blank
        // Cells the arity still claims. Pending and Invalid are not told
        // apart here — `atom: None` covers both.
        let grid = Grid::new(6, 1);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, ".+01  ");
        let frame = derive_frame(&source, grid.origin());

        let bound = frame.at(grid.position(2, 0).unwrap()).claim().expect("01");
        assert_eq!(bound.cells, 2..4);
        assert_eq!(bound.token, Token::Number);
        assert_eq!(bound.atom, Some(lang::Atom::Number(1)));
        assert!(std::ptr::eq(
            bound,
            frame.at(grid.position(3, 0).unwrap()).claim().expect("01")
        ));

        let pending = frame
            .at(grid.position(4, 0).unwrap())
            .claim()
            .expect("blank operand");
        assert_eq!(pending.cells, 4..6);
        assert_eq!(pending.token, Token::Number);
        assert_eq!(pending.atom, None);
        assert!(std::ptr::eq(
            pending,
            frame
                .at(grid.position(5, 0).unwrap())
                .claim()
                .expect("blank operand")
        ));
        assert!((4..6).all(|column| {
            frame.at(grid.position(column, 0).unwrap()).source_paint()
                == SourcePaint::Operand {
                    token: Token::Number,
                    state: OperandState::Pending,
                }
        }));
    }

    #[test]
    fn a_partly_written_operand_is_invalid_on_every_cell() {
        let grid = Grid::new(4, 1);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, ".+0");
        let frame = derive_frame(&source, grid.origin());

        assert!((2..4).all(|column| {
            frame.at(grid.position(column, 0).unwrap()).source_paint()
                == SourcePaint::Operand {
                    token: Token::Number,
                    state: OperandState::Invalid,
                }
        }));
    }

    #[test]
    fn every_cell_of_a_claim_reads_the_written_answer_its_claim_was_built_with() {
        // One 8-Cell row per shape a claim takes:
        //   `.+01  **` a bound Function, a Valid Number, a blank Pending
        //              Number that ends where the next Expression — a Bang —
        //              begins, and that Bang;
        //   `.+c40   ` a wholly written Invalid Number, then a partly
        //              written one;
        //   `|| a  b ` one Comment over the whole row, blanks included;
        //   `x       ` a refused Function spelling, then empty Cells.
        let grid = Grid::new(8, 4);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, ".+01  **.+c40   || a  b x       ");
        let frame = derive_frame(&source, grid.origin());

        let number = |state| SourcePaint::Operand {
            token: Token::Number,
            state,
        };
        let (valid, pending, invalid) = (
            number(OperandState::Valid),
            number(OperandState::Pending),
            number(OperandState::Invalid),
        );
        let (function, bang, comment, unclaimed) = (
            SourcePaint::Function,
            SourcePaint::Bang,
            SourcePaint::Comment,
            SourcePaint::Unclaimed,
        );
        let expected = [
            [
                function, function, valid, valid, pending, pending, bang, bang,
            ],
            [
                function, function, invalid, invalid, invalid, invalid, unclaimed, unclaimed,
            ],
            [comment; 8],
            [unclaimed; 8],
        ];

        for (y, row) in expected.iter().enumerate() {
            for (x, &answer) in row.iter().enumerate() {
                let drawn = frame.at(grid.position(x, y).unwrap());
                assert_eq!(drawn.source_paint(), answer, "column {x} row {y}");
                let Some(claim) = drawn.claim() else {
                    continue;
                };
                // Every Cell the claim covers shares it, so each reads the one
                // written answer the claim was built with.
                for index in claim.cells.clone() {
                    let covered = frame.at(grid.position_at(cell(grid, index)));
                    assert!(
                        covered
                            .claim()
                            .is_some_and(|other| std::ptr::eq(claim, other)),
                        "Cell {index} does not share the claim over column {x} row {y}"
                    );
                    assert_eq!(covered.source_paint(), answer, "Cell {index}");
                }
            }
        }
    }

    #[test]
    fn a_row_truncated_operand_cell_carries_its_declared_token() {
        // `.+01` written into a 5-wide Grid: an Add whose second Number
        // operand needs columns 4-5, and column 4 is the last column the
        // Grid has. The one Cell of that operand the Grid holds is empty
        // (the row simply ends there, left at its default space), and it
        // still carries `Token::Number` — `LanguageMap::token_at` already
        // reads this from the Parser's own record of the Cells the row's
        // tail held, so the Render Frame carries it through unchanged rather
        // than needing a second per-Cell classifier of its own.
        let grid = Grid::new(5, 1);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, ".+01");

        let frame = RenderFrame::derive(
            source.read_revision(),
            Region::at(grid, grid.origin()),
            false,
            RenderFrameConfig {
                sector_seam_spacing: SectorSeamSpacing::new(2).unwrap(),
                cursor_bloom_radius: CursorBloomRadius::new(1).unwrap(),
            },
        );

        let truncated = grid.position(4, 0).unwrap();
        assert_eq!(frame.at(truncated).content(), None);
        let claim = frame.at(truncated).claim().expect("truncated Number");
        assert_eq!(claim.cells, 4..5);
        assert_eq!(claim.token, Token::Number);
        assert_eq!(claim.atom, None);
    }

    #[test]
    fn a_lone_pipe_is_one_unbound_function_claim() {
        // A lone `|` is a refused Function spelling (ADR 0018): every unit
        // starts as a Function slot, the two-Cell read fails
        // `Function::try_from`, and the refusal advances one character.
        let grid = Grid::new(2, 1);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, "|");
        let frame = derive_frame(&source, grid.origin());

        let pipe = frame.at(grid.position(0, 0).unwrap()).claim().expect("|");
        assert_eq!(pipe.cells, 0..1);
        assert_eq!(pipe.token, Token::Function);
        assert_eq!(pipe.atom, None);
        assert!(
            frame.at(grid.position(1, 0).unwrap()).claim().is_none(),
            "the empty Cell beside `|` is unclaimed"
        );
    }

    #[test]
    fn a_written_07_is_two_one_cell_unbound_function_claims() {
        // A written `07` is two refused Function spellings (ADR 0018).
        // Nothing distinguishes `0`'s refusal from `7`'s — both are
        // `(Token::Function, None)` over one Cell.
        let grid = Grid::new(2, 1);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, "07");
        let frame = derive_frame(&source, grid.origin());

        let zero = frame.at(grid.position(0, 0).unwrap()).claim().expect("0");
        assert_eq!(zero.cells, 0..1);
        assert_eq!(zero.token, Token::Function);
        assert_eq!(zero.atom, None);

        let seven = frame.at(grid.position(1, 0).unwrap()).claim().expect("7");
        assert_eq!(seven.cells, 1..2);
        assert_eq!(seven.token, Token::Function);
        assert_eq!(seven.atom, None);
        assert!(
            !std::ptr::eq(zero, seven),
            "each refused Cell is its own claim"
        );
    }

    #[test]
    fn a_comment_is_one_claim_that_records_no_atom() {
        // A Comment records `Token::Comment` and no Atom (ADR 0035), the same
        // shape as an unbound entry, but it is a complete Language Unit. The
        // claim is one record shared across the introducer and the body.
        let grid = Grid::new(5, 1);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, "||abc");
        let frame = derive_frame(&source, grid.origin());

        let comment = frame
            .at(grid.position(0, 0).unwrap())
            .claim()
            .expect("Comment");
        assert_eq!(comment.cells, 0..5);
        assert_eq!(comment.token, Token::Comment);
        assert_eq!(comment.atom, None);
        for column in 1..5 {
            assert!(
                std::ptr::eq(
                    comment,
                    frame
                        .at(grid.position(column, 0).unwrap())
                        .claim()
                        .expect("Comment")
                ),
                "column {column}"
            );
        }
    }

    #[test]
    fn a_standalone_bang_is_one_bound_bang_claim() {
        let grid = Grid::new(2, 1);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, "**");
        let frame = derive_frame(&source, grid.origin());

        let bang = frame.at(grid.position(0, 0).unwrap()).claim().expect("**");
        assert_eq!(bang.cells, 0..2);
        assert_eq!(bang.token, Token::Bang);
        assert_eq!(bang.atom, Some(lang::Atom::Bang));
        assert!(std::ptr::eq(
            bang,
            frame.at(grid.position(1, 0).unwrap()).claim().expect("**")
        ));
    }

    #[test]
    fn an_unclaimed_cell_answers_no_claim() {
        let grid = Grid::new(2, 2);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, ".+");
        let frame = derive_frame(&source, grid.origin());

        assert!(
            frame.at(grid.position(0, 1).unwrap()).claim().is_none(),
            "an empty Cell no Expression covers is not a claim"
        );
    }

    #[test]
    fn occupied_glyphs_win_over_sector_presentation() {
        let grid = Grid::new(2, 2);
        let source = SourceCommander::new(grid);
        source.set(cell(grid, 0), "x").unwrap();

        let frame = RenderFrame::derive(
            source.read_revision(),
            Region::at(grid, grid.origin()),
            false,
            RenderFrameConfig {
                sector_seam_spacing: SectorSeamSpacing::new(1).unwrap(),
                cursor_bloom_radius: CursorBloomRadius::new(1).unwrap(),
            },
        );

        assert_eq!(cell_at(&frame, grid.origin()).content(), Some('x'));
        // A lone character is the first Cell of a spelling the Function table
        // does not hold, which is a classification like any other. What this
        // test is about is that it survives sector presentation at all.
        assert_eq!(token_at(&frame, grid.origin()), Some(Token::Function));
    }

    #[test]
    fn concurrent_ticks_cannot_mix_source_revisions_within_a_render_frame() {
        let grid = Grid::new(8, 2);
        let source = SourceCommander::new(grid);
        for (idx, content) in ".+010E".chars().enumerate() {
            source.set(cell(grid, idx), &content.to_string()).unwrap();
        }
        // Tick `0` of this Playback run, before either thread starts: it is
        // what puts a committed result in row 1 for the reader to observe.
        source.execute(Tick::ZERO);

        let start = Arc::new(Barrier::new(2));
        let writer_source = source.clone();
        let writer_start = start.clone();
        let writer = std::thread::spawn(move || {
            writer_start.wait();
            // ADR 0012 numbers each Tick after the first one on from the last,
            // so the writer carries the run forward from Tick `1` rather than
            // re-running Tick `0` two thousand times. What the reader is
            // watching for is a torn Render Frame, and a Playback run this
            // test could not otherwise describe is no basis for pinning one.
            let mut tick = Tick::ZERO.next();
            for operand in ['F', 'E'].into_iter().cycle().take(2_000) {
                writer_source
                    .set(cell(grid, 5), &operand.to_string())
                    .unwrap();
                writer_source.execute(tick);
                tick = tick.next();
            }
        });

        start.wait();
        for _ in 0..2_000 {
            let frame = RenderFrame::derive(
                source.read_revision(),
                Region::at(grid, grid.origin()),
                false,
                RenderFrameConfig {
                    sector_seam_spacing: SectorSeamSpacing::new(8).unwrap(),
                    cursor_bloom_radius: CursorBloomRadius::new(2).unwrap(),
                },
            );
            let result = (
                cell_at(&frame, grid.position(0, 1).unwrap()).content(),
                cell_at(&frame, grid.position(1, 1).unwrap()).content(),
            );
            assert!(
                result == (Some('0'), Some('F')) || result == (Some('1'), Some('0')),
                "one Render Frame mixed two Source revisions: {result:?}"
            );
        }
        writer.join().unwrap();
    }

    #[test]
    fn render_frame_answers_the_presentation_spacings_it_was_derived_with() {
        let grid = Grid::new(2, 2);
        let source = SourceCommander::new(grid);
        let sector_seam_spacing = SectorSeamSpacing::new(3).unwrap();
        let cursor_bloom_radius = CursorBloomRadius::new(5).unwrap();

        let frame = RenderFrame::derive(
            source.read_revision(),
            Region::at(grid, grid.origin()),
            false,
            RenderFrameConfig {
                sector_seam_spacing,
                cursor_bloom_radius,
            },
        );

        assert_eq!(frame.sector_seam_spacing(), sector_seam_spacing);
        assert_eq!(frame.cursor_bloom_radius(), cursor_bloom_radius);
    }

    #[test]
    fn render_frame_carries_the_region_it_was_derived_for() {
        let grid = Grid::new(4, 3);
        let source = SourceCommander::new(grid);
        let at = |x, y| grid.position(x, y).expect("inside the Grid");
        let region = Region::span(grid, at(3, 2), at(1, 0));

        let frame = RenderFrame::derive(
            source.read_revision(),
            region,
            false,
            RenderFrameConfig {
                sector_seam_spacing: SectorSeamSpacing::new(2).unwrap(),
                cursor_bloom_radius: CursorBloomRadius::new(1).unwrap(),
            },
        );

        assert_eq!(frame.region(), region);
        // The Cursor is the Region's live end, not its top-left.
        assert_eq!(frame.cursor(), at(1, 0));
    }

    fn write_row(source: &SourceCommander, grid: Grid, text: &str) {
        for (index, content) in text.chars().enumerate() {
            source.set(cell(grid, index), &content.to_string()).unwrap();
        }
    }

    fn derive_frame(source: &SourceCommander, selected: crate::grid::Position) -> RenderFrame {
        RenderFrame::derive(
            source.read_revision(),
            Region::at(source.grid(), selected),
            false,
            RenderFrameConfig {
                sector_seam_spacing: SectorSeamSpacing::new(2).unwrap(),
                cursor_bloom_radius: CursorBloomRadius::new(1).unwrap(),
            },
        )
    }

    #[test]
    fn a_parse_error_reaches_the_render_frame_for_the_expression_span() {
        // `.+0102` is a complete Add. `.+01` is the same Function one operand
        // short; the Language Map reports "expected a token" across that Span.
        let complete_grid = Grid::new(6, 1);
        let complete = SourceCommander::new(complete_grid);
        write_row(&complete, complete_grid, ".+0102");
        let complete_frame = derive_frame(&complete, complete_grid.origin());

        for position in complete_grid.positions_by_row().flatten() {
            assert!(
                !complete_frame.diagnostic_covers(position),
                "a complete Add does not diagnose Cell ({}, {})",
                position.x(),
                position.y()
            );
        }
        assert!(
            complete_frame
                .expression_at(complete_grid.origin())
                .expect("the Add covers the origin")
                .diagnostic()
                .is_none()
        );

        let incomplete_grid = Grid::new(4, 1);
        let incomplete = SourceCommander::new(incomplete_grid);
        write_row(&incomplete, incomplete_grid, ".+01");
        let incomplete_frame = derive_frame(&incomplete, incomplete_grid.origin());

        let expected = vec![
            incomplete_grid.position(0, 0).unwrap(),
            incomplete_grid.position(1, 0).unwrap(),
            incomplete_grid.position(2, 0).unwrap(),
            incomplete_grid.position(3, 0).unwrap(),
        ];
        let expression = incomplete_frame
            .expression_at(incomplete_grid.origin())
            .expect("the incomplete Add covers the origin");
        assert_eq!(expression.span().positions().collect::<Vec<_>>(), expected);
        let diagnostic = expression.diagnostic().expect("the incomplete Add reports");
        assert_eq!(diagnostic.message, "expected a token");
        assert_eq!(diagnostic.span().positions().collect::<Vec<_>>(), expected);
        for position in expected {
            assert!(incomplete_frame.diagnostic_covers(position));
        }
    }

    #[test]
    fn an_incomplete_function_is_not_executable_on_the_render_frame() {
        // A complete Add keeps its Function root. The same spelling one
        // operand short has no root: it will not run on the next Tick.
        let complete_grid = Grid::new(6, 1);
        let complete = SourceCommander::new(complete_grid);
        write_row(&complete, complete_grid, ".+0102");
        let complete_frame = derive_frame(&complete, complete_grid.origin());

        assert_eq!(
            complete_frame
                .expression_at(complete_grid.origin())
                .expect("the Add covers the origin")
                .root(),
            complete_grid.position(0, 0)
        );

        let incomplete_grid = Grid::new(4, 1);
        let incomplete = SourceCommander::new(incomplete_grid);
        write_row(&incomplete, incomplete_grid, ".+01");
        let incomplete_frame = derive_frame(&incomplete, incomplete_grid.origin());

        assert_eq!(
            incomplete_frame
                .expression_at(incomplete_grid.origin())
                .expect("the incomplete Add covers the origin")
                .root(),
            None
        );
    }

    #[test]
    fn a_lexical_diagnostic_reaches_a_cell_that_is_not_inside_an_expression() {
        // `***`: a Bang occupies cells 0-1. The leftover `*` is unmatched and
        // is not a Cell of that Expression; LanguageMap diagnoses it lexically
        // on its own Cell.
        let grid = Grid::new(3, 1);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, "***");
        let frame = derive_frame(&source, grid.origin());

        let leftover = grid.position(2, 0).unwrap();
        let bang = frame
            .expression_at(grid.origin())
            .expect("the Bang covers the origin");
        assert_eq!(
            bang.span().positions().collect::<Vec<_>>(),
            vec![grid.position(0, 0).unwrap(), grid.position(1, 0).unwrap(),]
        );
        assert!(
            bang.span().positions().all(|position| position != leftover),
            "the leftover Cell is not inside the Bang Expression"
        );

        let lexical = frame
            .lexical_diagnostics()
            .iter()
            .find(|diagnostic| {
                diagnostic
                    .span()
                    .positions()
                    .any(|position| position == leftover)
            })
            .expect("a lexical diagnostic covers the leftover Cell");
        assert_eq!(lexical.message, "invalid Language Unit character '*'");
        assert_eq!(
            lexical.span().positions().collect::<Vec<_>>(),
            vec![leftover]
        );
        assert!(frame.diagnostic_covers(leftover));
    }
}
