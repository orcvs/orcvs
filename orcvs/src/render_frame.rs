use std::sync::Arc;

use crate::{
    grid::{Grid, Position},
    opts::{CursorBloomRadius, SectorSeamSpacing},
    region::Region,
    source::{
        Claim, Diagnostic, Freshness, LockVerdict, OperandState, PortalEvent, PortalOrigin,
        PortalUse, SourcePaint, SourceRevision, Span, Token,
    },
};
use lang::{Atom, Function};

#[derive(Clone, Copy, Debug)]
pub(crate) struct RenderFrameConfig {
    pub sector_seam_spacing: SectorSeamSpacing,
    pub cursor_bloom_radius: CursorBloomRadius,
}

///
/// One part a Cell can play in a Portal.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortalRole {
    /// The Cell is where a Function writes its answer.
    Output,
    /// The Cell is where a Function reads an input.
    Input,
    /// The Cell spells the Function of an Expression root a Halt locked.
    Locked,
}

///
/// The set of [`PortalRole`]s one Cell plays: none, any one, or several.
///
/// ```
/// use orcvs::render_frame::{PortalRole, PortalRoles};
///
/// let roles = PortalRoles::NONE.with(PortalRole::Output);
/// assert!(roles.contains(PortalRole::Output));
/// assert!(!roles.contains(PortalRole::Input));
/// assert!(!roles.contains(PortalRole::Locked));
/// assert_eq!(roles.with(PortalRole::Output), roles);
/// ```
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PortalRoles(u8);

impl PortalRoles {
    /// The Cell plays no part in any Portal.
    pub const NONE: Self = Self(0);

    const fn bit(role: PortalRole) -> u8 {
        match role {
            PortalRole::Output => 1,
            PortalRole::Input => 1 << 1,
            PortalRole::Locked => 1 << 2,
        }
    }

    /// This set with `role` added.
    #[must_use]
    pub const fn with(self, role: PortalRole) -> Self {
        Self(self.0 | Self::bit(role))
    }

    /// Whether the Cell plays `role`.
    pub const fn contains(self, role: PortalRole) -> bool {
        self.0 & Self::bit(role) != 0
    }
}

#[derive(Clone, Debug)]
pub struct RenderCell {
    position: Position,
    content: Option<char>,
    claim: Option<Arc<Claim>>,
    portal_roles: PortalRoles,
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
            // The Bang Function is the one Function painted as what it
            // stands for rather than as a Function.
            Token::Function if claim.atom == Some(Atom::Function(Function::Bang)) => {
                SourcePaint::Bang
            }
            Token::Comment => SourcePaint::Comment,
            Token::Function if claim.atom.is_some() => SourcePaint::Function,
            Token::Function => SourcePaint::Unclaimed,
            Token::Char => unreachable!("the Parser never creates a positioned Char claim"),
            // An untyped slot has no type of its own, so its Cells paint as
            // plain Source text whatever they spell.
            Token::Untyped => SourcePaint::Unclaimed,
            token @ (Token::Number | Token::Note) => SourcePaint::Operand {
                token,
                state: if claim.atom.is_some() {
                    OperandState::Valid
                } else if claim.written {
                    OperandState::Invalid
                } else {
                    OperandState::Pending
                },
            },
        }
    }

    ///
    /// The Portal roles this Cell plays.
    ///
    /// [`PortalRole::Output`] covers the Cell pair from a value Function's
    /// Output Portal, the Reservation the Tick scheduler reserves for that
    /// Function's answer, whether it is a root or nested. That pair is
    /// derived from the current Source revision alone and known before any
    /// Tick runs. A Terminal Output Function, Halt, a Source-writing Function
    /// (including an Advance's cleared anchor), and a nested Function the
    /// Parser refuses reserve none, and neither does a scalar destination the
    /// row edge leaves no room for.
    ///
    /// It also covers each Cell a write of this revision's Tick Portals was
    /// admitted to, declared or selected. [`PortalRole::Input`] covers each
    /// Cell a read of them resolved, whatever the Cells held.
    /// [`PortalRole::Locked`] covers the two Cells of the Function of each
    /// Expression root a lock they record withheld. A refused access, a
    /// lock that withheld no root and a Source Bundle part set none. Roles
    /// combine: which of them a Cell shows is the console's decision.
    ///
    /// A [`Freshness::Retained`] record sets a role only for an entry
    /// whose Function still stands at its anchor in this revision, and whose
    /// subtree at the commit ([`PortalUse::subtree`]) holds no Cell an edit
    /// has written since. An entry whose operands selected its place also
    /// needs every read the record holds from one of that Function's
    /// descendants to have resolved no Cell an edit has written since,
    /// because such a read answers an operand's value. An edit to the
    /// Function, to one of its operands, or to a Cell a read nested in it
    /// resolved puts out that Function's entries until the next Tick, and an
    /// edit anywhere else, the rest of its Expression included, leaves them
    /// lit. A lock also needs no edit to have written the Cells it marks, so
    /// overwriting the locked root's spelling puts it out, and editing that
    /// root's operands leaves it marked. A Source no Tick has committed sets no role from a record.
    ///
    pub fn portal_roles(&self) -> PortalRoles {
        self.portal_roles
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
        let mut cells: Vec<RenderCell> = grid
            .positions_by_row()
            .flatten()
            .map(|position| {
                let index = grid.index(position).get();
                RenderCell {
                    position,
                    content: source.content_at(position),
                    claim: claims[index].clone(),
                    portal_roles: if output_portals[index] {
                        PortalRoles::NONE.with(PortalRole::Output)
                    } else {
                        PortalRoles::NONE
                    },
                }
            })
            .collect();
        // In place over the Cells just built, so a record costs no allocation.
        // A refused access carries no Span, and a lock that locked nothing
        // names no root, so neither has anything to paint. Source Bundles stay
        // in the record and paint nothing.
        if let Some(portals) = source.tick_portals() {
            // Only a retained record has edited reads. The highlight is in the
            // Cells now, so its storage marks the anchor of each recorded read
            // an edit has written a Cell of, before any entry asks whether one
            // of its descendants made one. With none marked, no entry walks its
            // subtree for one.
            let edited_reads = (portals.freshness() == Freshness::Retained).then(|| {
                let mut marks = output_portals;
                marks.fill(false);
                let mut any = false;
                for access in portals.record().uses() {
                    if let PortalEvent::Read { site: Ok(read), .. } = access.event()
                        && source.span_edited(read.span())
                    {
                        marks[grid.index(access.anchor()).get()] = true;
                        any = true;
                    }
                }
                any.then_some(marks)
            });
            for access in portals.record().uses() {
                let (span, role, place) = match access.event() {
                    PortalEvent::Read {
                        site: Ok(read),
                        origin,
                    } => (read.span(), PortalRole::Input, Place::Portal(*origin)),
                    PortalEvent::Write {
                        outcome: Ok(written),
                        origin,
                    } => (written.span(), PortalRole::Output, Place::Portal(*origin)),
                    // A root that stood at its anchor fits its spelling in its
                    // row, so a lock marks nothing only where no root stood.
                    PortalEvent::Lock {
                        verdict: LockVerdict::Locked(root),
                        ..
                    } => match Span::of_function(grid, *root) {
                        Some(spelling) => (spelling, PortalRole::Locked, Place::Lock),
                        None => continue,
                    },
                    PortalEvent::Read { site: Err(_), .. }
                    | PortalEvent::Write {
                        outcome: Err(_), ..
                    }
                    | PortalEvent::Lock { .. }
                    | PortalEvent::Bundle { .. } => continue,
                };
                if let Some(edited_reads) = &edited_reads
                    && !(owner_stands(grid, &claims, access)
                        && !source.span_edited(access.subtree())
                        && match place {
                            Place::Portal(PortalOrigin::Declared) => true,
                            Place::Portal(PortalOrigin::Selected) => {
                                edited_reads.as_deref().is_none_or(|edited_reads| {
                                    !descendant_read_edited(grid, edited_reads, access)
                                })
                            }
                            Place::Lock => !source.span_edited(span),
                        })
                {
                    continue;
                }
                for position in span.positions() {
                    let cell = &mut cells[grid.index(position).get()];
                    cell.portal_roles = cell.portal_roles.with(role);
                }
            }
        }
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
    pub fn grid(&self) -> Grid {
        self.grid
    }

    ///
    /// The Cursor of [`Self::region`], which the derivation asserted the Grid
    /// owns.
    ///
    pub fn cursor(&self) -> Position {
        self.region.cursor()
    }

    ///
    /// The Region the derivation was given: from the anchor to its live end,
    /// with the Cursor on one of its Cells.
    ///
    /// It is running state rather than Source, so it is never stored with
    /// one.
    ///
    pub fn region(&self) -> Region {
        self.region
    }

    ///
    /// Whether the Cursor is visible in this Frame.
    ///
    /// Visibility is a fact about the Frame, not about any one Cell.
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

///
/// How a recorded entry found the Cells it marks, which decides what else a
/// retained entry needs, beyond its owner standing unedited, to mark them.
///
#[derive(Clone, Copy)]
enum Place {
    /// A read or a write, through a Portal its Function declares or one its
    /// operands selected. A declared place needs nothing more; a selected one
    /// needs no read nested in its owner to have read an edited Cell.
    Portal(PortalOrigin),
    /// A lock, at the fixed offset a Halt declares. It marks the locked
    /// root's own spelling, so it needs no edit to have written those Cells.
    Lock,
}

///
/// Whether the Function whose Turn made `access` still stands at its anchor
/// in the revision `claims` were read from: the Language Map claims the
/// anchor's Cell for an entry that begins there and parsed as that Function,
/// which is how a schedule anchors the computation that runs it.
///
fn owner_stands(grid: Grid, claims: &[Option<Arc<Claim>>], access: &PortalUse) -> bool {
    let anchor = grid.index(access.anchor()).get();
    claims[anchor].as_deref().is_some_and(|claim| {
        claim.cells.start == anchor && claim.atom == Some(Atom::Function(access.function()))
    })
}

///
/// Whether `edited_reads`, marked at the anchor of each recorded read an edit
/// has since written a Cell of, marks a descendant of the Function whose Turn
/// made `access`: an anchor in the Cells its subtree holds, other than its own.
///
fn descendant_read_edited(grid: Grid, edited_reads: &[bool], access: &PortalUse) -> bool {
    access
        .subtree()
        .positions()
        .any(|position| position != access.anchor() && edited_reads[grid.index(position).get()])
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
        render_frame::{PortalRole, PortalRoles, RenderFrame, RenderFrameConfig},
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
    /// claims it. A Render Cell answers its Token only through `claim()`.
    ///
    fn token_at(frame: &RenderFrame, position: crate::grid::Position) -> Option<Token> {
        frame.at(position).claim().map(|claim| claim.token)
    }

    #[test]
    fn render_frame_is_a_complete_row_structured_visual_snapshot() {
        let grid = Grid::with_shape(2, 2);
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
        let grid = Grid::with_shape(4, 1);
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

        for x in 0..2 {
            let position = grid.position(x, 0).unwrap();
            assert_eq!(token_at(&frame, position), Some(Token::Function));
            assert_eq!(frame.at(position).source_paint(), SourcePaint::Bang);
        }
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
        let grid = Grid::with_shape(2, 1);
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

        // `>>` is a row of the Function table, so it is painted where every
        // other Function is, not as leftover Char: these two Cells spell a
        // Function.
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
        let grid = Grid::with_shape(6, 1);
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
    fn a_writes_value_paints_as_plain_source_text() {
        let grid = Grid::with_shape(6, 1);
        for row in ["@>010A", "@>01G4", "@>01C4", "@>01**", "@>01  ", "@>01xy"] {
            let source = SourceCommander::new(grid);
            write_row(&source, grid, row);
            let frame = derive_frame(&source, grid.origin());
            for column in 4..6 {
                assert_eq!(
                    frame.at(grid.position(column, 0).unwrap()).source_paint(),
                    SourcePaint::Unclaimed,
                    "{row:?} column {column}",
                );
            }
        }
    }

    #[test]
    fn a_bound_number_sits_beside_an_unbound_blank_operand_claim() {
        // `.+01  `: the first Number binds; the second operand is two blank
        // Cells the arity still claims. `atom: None` covers both Pending and
        // Invalid; the claim's `written` answer makes these blank Cells
        // Pending.
        let grid = Grid::with_shape(6, 1);
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
        let grid = Grid::with_shape(4, 1);
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
        //   `|| a  b ` one unclosed Comment over the whole row, blanks included;
        //   `x       ` a refused Function spelling, then empty Cells.
        let grid = Grid::with_shape(8, 4);
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
        let grid = Grid::with_shape(5, 1);
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
        // A lone `|` is a refused Function spelling: every unit starts as a
        // Function slot, the two-Cell read fails `Function::try_from`, and the
        // refusal advances one character.
        let grid = Grid::with_shape(2, 1);
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
        // A written `07` is two refused Function spellings. Nothing
        // distinguishes `0`'s refusal from `7`'s — both are
        // `(Token::Function, None)` over one Cell.
        let grid = Grid::with_shape(2, 1);
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
        // A Comment records `Token::Comment` and no Atom, the same
        // shape as an unbound entry, but it is a complete Language Unit. The
        // claim is one record shared across the introducer and the body.
        let grid = Grid::with_shape(5, 1);
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
        let grid = Grid::with_shape(2, 1);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, "**");
        let frame = derive_frame(&source, grid.origin());

        let bang = frame.at(grid.position(0, 0).unwrap()).claim().expect("**");
        assert_eq!(bang.cells, 0..2);
        assert_eq!(bang.token, Token::Function);
        assert_eq!(bang.atom, Some(lang::Atom::Function(lang::Function::Bang)));
        assert!(std::ptr::eq(
            bang,
            frame.at(grid.position(1, 0).unwrap()).claim().expect("**")
        ));
    }

    #[test]
    fn an_unclaimed_cell_answers_no_claim() {
        let grid = Grid::with_shape(2, 2);
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
        let grid = Grid::with_shape(2, 2);
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
        let grid = Grid::with_shape(8, 2);
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
        let grid = Grid::with_shape(2, 2);
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
        let grid = Grid::with_shape(4, 3);
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
        let complete_grid = Grid::with_shape(6, 1);
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

        let incomplete_grid = Grid::with_shape(4, 1);
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
        let complete_grid = Grid::with_shape(6, 1);
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

        let incomplete_grid = Grid::with_shape(4, 1);
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
        let grid = Grid::with_shape(3, 1);
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

    #[test]
    fn each_cell_plays_the_output_role_its_revision_highlights_and_no_input_role() {
        let grid = Grid::with_shape(8, 2);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, ".+0304");
        let revision = source.read_revision();
        let highlight = revision.output_portal_highlight();
        let frame = derive_frame(&source, grid.origin());

        assert!(
            highlight.iter().any(|covered| *covered),
            "the Add reserves an Output Portal, so the comparison is not vacuous"
        );
        for position in grid.positions_by_row().flatten() {
            let roles = frame.at(position).portal_roles();
            assert_eq!(
                roles.contains(PortalRole::Output),
                highlight[grid.index(position).get()],
                "({}, {})",
                position.x(),
                position.y()
            );
            assert!(!roles.contains(PortalRole::Input));
        }
    }

    /// The Cells of `frame` that play the Input role, by column and row.
    fn input_cells(frame: &RenderFrame) -> Vec<(usize, usize)> {
        frame
            .cells()
            .iter()
            .filter(|cell| cell.portal_roles().contains(PortalRole::Input))
            .map(|cell| (cell.position().x(), cell.position().y()))
            .collect()
    }

    #[test]
    fn each_cell_a_committed_tick_read_plays_the_input_role_and_moves_with_the_read() {
        // `&v` reads the pair `n` rows south of its operand: (2, 1) at `01`,
        // and (2, 2) once `n` is `02`.
        let grid = Grid::with_shape(4, 4);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, "&v01");
        assert_eq!(input_cells(&derive_frame(&source, grid.origin())), []);

        source.execute(Tick::ZERO);
        assert_eq!(
            input_cells(&derive_frame(&source, grid.origin())),
            [(2, 1), (3, 1)]
        );

        source.set(cell(grid, 3), "2").unwrap();
        source.execute(Tick::new(1));
        assert_eq!(
            input_cells(&derive_frame(&source, grid.origin())),
            [(2, 2), (3, 2)]
        );
    }

    #[test]
    fn a_cell_a_committed_tick_both_read_and_wrote_plays_both_roles() {
        // `@$` writes `C4` at (2, 1), and the Read at (8, 0) then reads it.
        // Neither pair is a static Output Portal, so only the record lights
        // them.
        let grid = Grid::with_shape(16, 3);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, "@$0201C4&$0201");
        source.execute(Tick::ZERO);
        let frame = derive_frame(&source, grid.origin());
        for x in [2, 3] {
            let roles = cell_at(&frame, grid.position(x, 1).unwrap()).portal_roles();
            assert!(roles.contains(PortalRole::Output), "({x}, 1): {roles:?}");
            assert!(roles.contains(PortalRole::Input), "({x}, 1): {roles:?}");
        }
    }

    /// The Cells of `frame` that play the Output role, by column and row.
    fn output_cells(frame: &RenderFrame) -> Vec<(usize, usize)> {
        frame
            .cells()
            .iter()
            .filter(|cell| cell.portal_roles().contains(PortalRole::Output))
            .map(|cell| (cell.position().x(), cell.position().y()))
            .collect()
    }

    #[test]
    fn after_an_edit_only_the_accesses_whose_function_still_stands_at_its_anchor_paint() {
        // Two voices: `&v01` at (0, 0) reads (2, 1) and (3, 1), and `&v02`
        // at (4, 0) reads (6, 2) and (7, 2).
        let grid = Grid::with_shape(8, 4);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, "&v01&v02");
        source.execute(Tick::ZERO);
        assert_eq!(
            input_cells(&derive_frame(&source, grid.origin())),
            [(2, 1), (3, 1), (6, 2), (7, 2)]
        );
        // Each voice writes what it reads one row south of its anchor.
        assert_eq!(
            output_cells(&derive_frame(&source, grid.origin())),
            [(0, 1), (1, 1), (4, 1), (5, 1)]
        );

        // The second voice is now a Read North, which never made the read
        // the record holds, so only the first voice's read paints: the edit
        // stands outside the first voice's Expression.
        source.set(cell(grid, 5), "^").unwrap();
        assert_eq!(
            input_cells(&derive_frame(&source, grid.origin())),
            [(2, 1), (3, 1)]
        );

        // With no Function at its anchor, the first voice claims nothing:
        // neither its read nor the write the record holds at (0, 1) and
        // (1, 1). The Read North still reserves its own Output Portal.
        source.unset(cell(grid, 0));
        let frame = derive_frame(&source, grid.origin());
        assert_eq!(input_cells(&frame), []);
        assert_eq!(output_cells(&frame), [(4, 1), (5, 1)]);
    }

    #[test]
    fn a_paused_edit_inside_an_owners_expression_puts_out_its_accesses_and_no_others() {
        // Two voices: `&v01` at (0, 0) reads (2, 1) and (3, 1), and `&v02`
        // at (4, 0) reads (6, 2) and (7, 2). No Tick runs after the edits.
        let grid = Grid::with_shape(8, 4);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, "&v01&v02");
        source.execute(Tick::ZERO);

        // The first voice's operand now selects (2, 2): the Function still
        // stands, but the record no longer says what it reads or writes.
        source.set(cell(grid, 3), "2").unwrap();
        let frame = derive_frame(&source, grid.origin());
        assert_eq!(input_cells(&frame), [(6, 2), (7, 2)]);
        assert_eq!(output_cells(&frame), [(0, 1), (1, 1), (4, 1), (5, 1)]);

        // Writing back what the commit left is still an edit of the first
        // voice, so it stays out until the next Tick, and an edit outside
        // every owner's Expression puts out nothing.
        source.set(cell(grid, 3), "1").unwrap();
        source.set(cell(grid, 24), "x").unwrap();
        assert_eq!(
            input_cells(&derive_frame(&source, grid.origin())),
            [(6, 2), (7, 2)]
        );

        // The next Tick's record lights both voices again.
        source.execute(Tick::new(1));
        assert_eq!(
            input_cells(&derive_frame(&source, grid.origin())),
            [(2, 1), (3, 1), (6, 2), (7, 2)]
        );
    }

    #[test]
    fn a_paused_edit_to_a_cell_a_nested_read_selects_from_puts_out_the_outer_read() {
        // `&v&v01` at (0, 0): the inner Read at (2, 0) reads `01` from (4, 1)
        // and (5, 1), and that value is the outer Read's operand, so the
        // outer Read reads (2, 1) and (3, 1). No Tick runs after the edit.
        let grid = Grid::with_shape(8, 4);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, "&v&v01");
        source.set(cell(grid, 12), "0").unwrap();
        source.set(cell(grid, 13), "1").unwrap();
        source.execute(Tick::ZERO);
        assert_eq!(
            input_cells(&derive_frame(&source, grid.origin())),
            [(2, 1), (3, 1), (4, 1), (5, 1)]
        );

        // (5, 1) stands outside the Expression, but the inner Read now reads
        // `02`, so the outer Read selects (2, 2): the record no longer says
        // what it reads, and only the inner Read's read stays lit.
        source.set(cell(grid, 13), "2").unwrap();
        assert_eq!(
            input_cells(&derive_frame(&source, grid.origin())),
            [(4, 1), (5, 1)]
        );
    }

    #[test]
    fn a_paused_edit_inside_a_chain_of_nested_reads_puts_out_every_read_it_feeds() {
        // `&v&v&v01` at (0, 0): the innermost Read at (4, 0) reads `01` from
        // (6, 1) and writes it to (4, 1), where the middle Read at (2, 0)
        // reads it and writes it to (2, 1), where the outer Read reads it.
        let grid = Grid::with_shape(8, 4);
        let committed = || {
            let source = SourceCommander::new(grid);
            write_row(&source, grid, "&v&v&v01");
            source.set(cell(grid, 14), "0").unwrap();
            source.set(cell(grid, 15), "1").unwrap();
            source.execute(Tick::ZERO);
            assert_eq!(
                input_cells(&derive_frame(&source, grid.origin())),
                [(2, 1), (3, 1), (4, 1), (5, 1), (6, 1), (7, 1)]
            );
            source
        };

        // The middle Read's Cells feed only the outer Read.
        let source = committed();
        source.set(cell(grid, 13), "2").unwrap();
        assert_eq!(
            input_cells(&derive_frame(&source, grid.origin())),
            [(4, 1), (5, 1), (6, 1), (7, 1)]
        );

        // The innermost Read's Cells feed both Reads it is nested in.
        let source = committed();
        source.set(cell(grid, 15), "2").unwrap();
        assert_eq!(
            input_cells(&derive_frame(&source, grid.origin())),
            [(6, 1), (7, 1)]
        );
    }

    #[test]
    fn a_paused_edit_to_one_reads_cells_leaves_a_sibling_read_lit() {
        // `.+&v01&v02` at (0, 0): the Reads at (2, 0) and (6, 0) are both
        // operands of the Add, and neither answers an operand of the other.
        let grid = Grid::with_shape(16, 4);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, ".+&v01&v02");
        source.execute(Tick::ZERO);
        let committed = [(4, 1), (5, 1), (8, 2), (9, 2)];
        assert_eq!(
            input_cells(&derive_frame(&source, grid.origin())),
            committed
        );

        // The second Read's Cells feed the Add, not the first Read, and a
        // Read's own entries stay lit when the Cells it read are edited.
        source.set(cell(grid, 41), "3").unwrap();
        assert_eq!(
            input_cells(&derive_frame(&source, grid.origin())),
            committed
        );
    }

    #[test]
    fn a_paused_edit_elsewhere_in_an_expression_leaves_the_reads_it_does_not_feed_lit() {
        // `.+&v01&v02` at (0, 0). Rewriting the Add as a Subtract changes
        // neither Read's operands, so both still read where they read.
        let grid = Grid::with_shape(16, 4);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, ".+&v01&v02");
        source.execute(Tick::ZERO);
        let committed = [(4, 1), (5, 1), (8, 2), (9, 2)];
        assert_eq!(
            input_cells(&derive_frame(&source, grid.origin())),
            committed
        );

        source.set(cell(grid, 1), "-").unwrap();
        assert_eq!(
            input_cells(&derive_frame(&source, grid.origin())),
            committed
        );
    }

    #[test]
    fn a_nested_read_whose_parent_is_edited_away_stays_lit_where_it_still_reads() {
        // `&v&v01` at (0, 0), with `01` at (4, 1). Overwriting the outer
        // Read's `&` leaves the inner Read standing as a root of its own, with
        // its own Cells unedited, so it still reads (4, 1) and (5, 1). The
        // outer Read no longer stands.
        let grid = Grid::with_shape(8, 4);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, "&v&v01");
        source.set(cell(grid, 12), "0").unwrap();
        source.set(cell(grid, 13), "1").unwrap();
        source.execute(Tick::ZERO);

        source.set(cell(grid, 0), "x").unwrap();
        assert_eq!(
            input_cells(&derive_frame(&source, grid.origin())),
            [(4, 1), (5, 1)]
        );
    }

    #[test]
    fn a_paused_edit_to_a_nested_reads_cells_leaves_a_declared_read_lit() {
        // `~+&v0110` at (0, 0): the Increment reads its previous value at the
        // Portal it declares, one row south, and its step is what the Read at
        // (2, 0) reads from (4, 1) and (5, 1).
        let grid = Grid::with_shape(8, 4);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, "~+&v0110");
        source.set(cell(grid, 12), "0").unwrap();
        source.set(cell(grid, 13), "1").unwrap();
        source.execute(Tick::ZERO);
        let committed = [(0, 1), (1, 1), (4, 1), (5, 1)];
        assert_eq!(
            input_cells(&derive_frame(&source, grid.origin())),
            committed
        );

        // The step changes, but no operand chose where the Increment reads.
        source.set(cell(grid, 13), "2").unwrap();
        assert_eq!(
            input_cells(&derive_frame(&source, grid.origin())),
            committed
        );
    }

    #[test]
    fn a_function_that_selects_nothing_between_two_reads_still_passes_the_edit_up() {
        // `&v.+&v0100` at (0, 0): the inner Read at (4, 0) reads `01` from
        // (6, 1), the Add at (2, 0) adds `00` to it, and the outer Read reads
        // where that sum selects, (2, 1). The Add's own place is declared.
        let grid = Grid::with_shape(10, 4);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, "&v.+&v0100");
        source.set(cell(grid, 16), "0").unwrap();
        source.set(cell(grid, 17), "1").unwrap();
        source.execute(Tick::ZERO);
        assert_eq!(
            input_cells(&derive_frame(&source, grid.origin())),
            [(2, 1), (3, 1), (6, 1), (7, 1)]
        );

        source.set(cell(grid, 17), "2").unwrap();
        assert_eq!(
            input_cells(&derive_frame(&source, grid.origin())),
            [(6, 1), (7, 1)]
        );
    }

    /// The Cells of `frame` that play the Locked role, by column and row.
    fn locked_cells(frame: &RenderFrame) -> Vec<(usize, usize)> {
        frame
            .cells()
            .iter()
            .filter(|cell| cell.portal_roles().contains(PortalRole::Locked))
            .map(|cell| (cell.position().x(), cell.position().y()))
            .collect()
    }

    /// Equality at (0, 0) bangs the Halt at (2, 1), which locks the Add at
    /// (2, 2) for the Tick.
    const HALTED_ADD: &str = ".=0101    *!      .+0304";

    #[test]
    fn the_cells_of_the_root_a_committed_tick_locked_play_the_locked_role() {
        let grid = Grid::with_shape(8, 4);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, HALTED_ADD);
        assert_eq!(locked_cells(&derive_frame(&source, grid.origin())), []);

        source.execute(Tick::ZERO);
        let frame = derive_frame(&source, grid.origin());
        assert_eq!(locked_cells(&frame), [(2, 2), (3, 2)]);
        // The lock is no read or write: the locked Add's Cells play no other
        // role, and its operands play none.
        for x in 2..8 {
            let roles = frame.at(grid.position(x, 2).unwrap()).portal_roles();
            assert!(!roles.contains(PortalRole::Input), "({x}, 2): {roles:?}");
            assert!(!roles.contains(PortalRole::Output), "({x}, 2): {roles:?}");
        }
    }

    #[test]
    fn a_lock_that_locked_nothing_marks_no_cell() {
        // With nothing at its site, the Halt's lock changes nothing.
        let grid = Grid::with_shape(8, 4);
        let source = SourceCommander::new(grid);
        write_row(&source, grid, ".=0101    *!");
        source.execute(Tick::ZERO);
        assert_eq!(locked_cells(&derive_frame(&source, grid.origin())), []);
    }

    #[test]
    fn a_retained_lock_stays_marked_until_an_edit_reaches_its_halt_or_its_root() {
        let grid = Grid::with_shape(8, 4);
        let committed = || {
            let source = SourceCommander::new(grid);
            write_row(&source, grid, HALTED_ADD);
            source.execute(Tick::ZERO);
            source
        };

        // The lock's site is a fixed offset from the Halt, so an edit outside
        // the Halt's own Cells and the Cells it marks, the locked root's
        // operands included, leaves the mark standing.
        let source = committed();
        source.set(cell(grid, 31), "x").unwrap();
        source.set(cell(grid, 21), "5").unwrap();
        assert_eq!(
            locked_cells(&derive_frame(&source, grid.origin())),
            [(2, 2), (3, 2)]
        );

        // Overwriting either Cell of the locked root's own spelling leaves
        // the Cells the mark lands on edited, so the mark goes out until the
        // next Tick.
        for edited in [18, 19] {
            let source = committed();
            source.set(cell(grid, edited), "x").unwrap();
            assert_eq!(
                locked_cells(&derive_frame(&source, grid.origin())),
                [],
                "Cell {edited}"
            );
        }

        // Overwriting the Halt leaves no Function standing at its anchor.
        let source = committed();
        source.set(cell(grid, 10), "x").unwrap();
        assert_eq!(locked_cells(&derive_frame(&source, grid.origin())), []);

        // Writing back what the commit left is still an edit of the Halt, so
        // the mark stays out until the next Tick commits a lock again.
        source.set(cell(grid, 10), "*").unwrap();
        assert_eq!(locked_cells(&derive_frame(&source, grid.origin())), []);
        source.execute(Tick::new(1));
        assert_eq!(
            locked_cells(&derive_frame(&source, grid.origin())),
            [(2, 2), (3, 2)]
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn deriving_with_tick_portals_allocates_no_more_than_without() {
        use crate::source::Freshness;

        // Two Sources holding the same Cells, one whose revision a Tick
        // committed with a record of reads, writes, one of them selected, and
        // a lock, and one written by hand with none, so every other input to
        // the derivation is the same. Emptying an empty Cell outside every
        // Expression then makes the record retained over the same Cells, with
        // an edited Cell marked, so every painted entry asks whether its
        // Function still stands, its subtree is unedited, and, for a selected
        // place, no read nested in it read an edited Cell.
        let grid = Grid::with_shape(16, 4);
        let ticked = SourceCommander::new(grid);
        write_row(&ticked, grid, "&v01&v02@$0E03C4");
        // Equality at (8, 1) bangs the Halt at (10, 2), which locks the Add
        // at (10, 3).
        for (index, content) in [(24, ".=0101"), (42, "*!"), (58, ".+0304")] {
            for (offset, content) in content.chars().enumerate() {
                ticked
                    .set(cell(grid, index + offset), &content.to_string())
                    .unwrap();
            }
        }
        ticked.execute(Tick::ZERO);
        let revision = ticked.read_revision();
        // Two reads, the two writes the Reads declare, the Write's, the
        // Equality's and the lock. The locked Add takes no Turn.
        assert_eq!(
            revision
                .tick_portals()
                .map(|portals| (portals.freshness(), portals.record().uses().len())),
            Some((Freshness::Current, 7))
        );
        ticked.unset(cell(grid, 48));
        let retained = ticked.read_revision();
        assert_eq!(
            retained.tick_portals().map(|portals| portals.freshness()),
            Some(Freshness::Retained)
        );

        let written = SourceCommander::new(grid);
        let cells: Vec<_> = ticked
            .snapshot()
            .bytes()
            .enumerate()
            .map(|(idx, byte)| crate::source::CellWrite {
                cell: cell(grid, idx),
                content: crate::source::CellContent::new(byte).expect("a Cell"),
            })
            .collect();
        written.write_cells(&cells);
        let bare = written.read_revision();
        assert_eq!(ticked.snapshot(), written.snapshot());
        assert!(bare.tick_portals().is_none());

        let derive = |revision| {
            RenderFrame::derive(
                revision,
                Region::at(grid, grid.origin()),
                false,
                RenderFrameConfig {
                    sector_seam_spacing: SectorSeamSpacing::new(2).unwrap(),
                    cursor_bloom_radius: CursorBloomRadius::new(1).unwrap(),
                },
            )
        };
        let (with_record, frame) = crate::allocation::blocks(|| derive(revision));
        assert_eq!(input_cells(&frame).len(), 4, "the reads were folded");
        assert!(
            cell_at(&frame, grid.position(14, 3).unwrap())
                .portal_roles()
                .contains(PortalRole::Output),
            "the Write was folded"
        );
        assert_eq!(
            locked_cells(&frame),
            [(10, 3), (11, 3)],
            "the lock was folded"
        );
        let (with_retained, frame) = crate::allocation::blocks(|| derive(retained));
        assert_eq!(
            input_cells(&frame).len(),
            4,
            "the retained reads were folded"
        );
        assert!(
            cell_at(&frame, grid.position(14, 3).unwrap())
                .portal_roles()
                .contains(PortalRole::Output),
            "the retained Write was folded"
        );
        assert_eq!(
            locked_cells(&frame),
            [(10, 3), (11, 3)],
            "the retained lock was folded"
        );
        let (without, _) = crate::allocation::blocks(|| derive(bare));
        assert!(
            with_record <= without,
            "{with_record} blocks with a record, {without} without"
        );
        assert!(
            with_retained <= without,
            "{with_retained} blocks with a retained record, {without} without"
        );
    }

    #[test]
    fn portal_roles_hold_each_role_independently() {
        let output = PortalRoles::NONE.with(PortalRole::Output);
        let input = PortalRoles::NONE.with(PortalRole::Input);
        let locked = PortalRoles::NONE.with(PortalRole::Locked);
        let both = output.with(PortalRole::Input);
        let all = both.with(PortalRole::Locked);

        for (roles, holds_output, holds_input, holds_locked) in [
            (PortalRoles::NONE, false, false, false),
            (output, true, false, false),
            (input, false, true, false),
            (locked, false, false, true),
            (both, true, true, false),
            (input.with(PortalRole::Locked), false, true, true),
            (all, true, true, true),
        ] {
            assert_eq!(
                roles.contains(PortalRole::Output),
                holds_output,
                "{roles:?}"
            );
            assert_eq!(roles.contains(PortalRole::Input), holds_input, "{roles:?}");
            assert_eq!(
                roles.contains(PortalRole::Locked),
                holds_locked,
                "{roles:?}"
            );
        }
        assert_eq!(both, input.with(PortalRole::Output));
        assert_eq!(both.with(PortalRole::Output), both);
        assert_eq!(all, locked.with(PortalRole::Input).with(PortalRole::Output));
    }
}
