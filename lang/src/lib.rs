mod atom;
mod error;
mod expression;
mod functions;
mod interpreter;
pub mod operand;
mod parser;
mod portal;
mod stack;
mod tick;

pub use atom::{
    Atom, Atoms, BendLsb, BendMsb, ControlValue, Controller, DistanceRule, Function, Length,
    MidiChannel, Note, PositionRule, ReplacementChange, Spelling, Velocity, to_atom_note,
    to_atom_num,
};
pub use error::{ArgumentError, Error, InterpretationError, SyntaxError, TypeError};
pub use expression::{CommentText, Expression, PositionedEntry, Token, Tokens};
pub use interpreter::{Interpretation, Interpreter};
pub use parser::{Parser, SourceAnalysis};
pub use portal::{FunctionInputs, PortalInput, PortalSource};
pub(crate) use stack::Stack;
pub use tick::{Anchor, Tick, TickInputs};

#[cfg(test)]
use std::sync::Once;

/// One interpreted MIDI instruction a Terminal Output Function emits for
/// delivery during a Tick.
///
/// This is a tagged variant set rather than one note triple because the
/// Terminal Output family is wider than Raw Play: Timed and Monophonic Play,
/// Control Change, and Pitch Bend each carry different validated data and
/// arrive here as variants of their own. Every field is a domain type rather
/// than a byte, so the check the emitting Function made travels with the value
/// and no consumer has to repeat it; and none holds wire bytes, because
/// assembling a MIDI message belongs to the output adapter and Source
/// interpretation never learns the protocol encoding. A field whose domain is
/// carried by a type also cannot be transposed with a field of another domain.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayCommand {
    /// Raw Play. Velocity `00` is not an absent note but the Source's
    /// explicit stop, which the Playback Engine delivers as a Note Off.
    Raw {
        channel: MidiChannel,
        velocity: Velocity,
        note: Note,
    },
    ///
    /// Timed Play, carrying the whole lifetime the Source wrote.
    ///
    /// The length is in the command rather than resolved here because a Note
    /// Off due at Tick `T + length` belongs to a Playback run: interpretation
    /// plans one Tick, and a Tick has nowhere to put an effect due at another
    /// one. Handing the length on is what lets the Playback Engine schedule
    /// the stop without inferring musical intent, which is the seam ADR 0001
    /// draws.
    ///
    Timed {
        channel: MidiChannel,
        velocity: Velocity,
        note: Note,
        length: Length,
    },
    ///
    /// Monophonic Play, carrying the same operands Timed Play does.
    ///
    /// A variant of its own rather than a flag on `Timed`, because the two
    /// differ in what they own rather than in what they carry: Timed Play is
    /// polyphonic and owns the note it names, while Monophonic Play owns its
    /// whole channel and replaces whatever that channel was sounding. A shared
    /// variant would put that difference in a field every consumer had to
    /// branch on, where a variant makes the match arms the branch.
    ///
    Mono {
        channel: MidiChannel,
        velocity: Velocity,
        note: Note,
        length: Length,
    },
    /// Control Change: a controller and the value sent to it, each
    /// carrying the role it plays rather than the data-byte domain the two
    /// share, so nothing downstream can put one where the other belongs.
    ControlChange {
        channel: MidiChannel,
        controller: Controller,
        value: ControlValue,
    },
    ///
    /// Pitch Bend, as the two seven-bit halves the wire carries.
    ///
    /// A bend is one fourteen-bit value, and Orcvs neither assembles the
    /// halves into it nor scales them: the Source writes the bytes MIDI sends,
    /// so there is nothing here to normalize and nothing for a consumer to
    /// take apart. The two halves are separate types for the reason Control
    /// Change's two data bytes are, with the wire order — LSB before MSB —
    /// riding on the distinction.
    ///
    PitchBend {
        channel: MidiChannel,
        lsb: BendLsb,
        msb: BendMsb,
    },
}

/// One Source-writing effect a Function performs, stated relative to the
/// producer's own anchor.
///
/// A Source-writing Function plans a validated Portal bundle, and ADR 0009
/// keeps destination resolution in `orcvs`, which owns the Grid. This type is
/// the seam between the two: `lang` answers what to write and how far from the
/// producer to write it, and `orcvs` turns that into Positions, refuses a
/// destination the Grid does not hold, and orders the writes. It is the
/// Source-writing counterpart of [`PlayCommand`], which crosses the same seam
/// for the Terminal Output family.
///
/// The displacement is a whole-Cell offset and not a named direction: ADR 0006
/// states the geometry in coordinates, and a Portal is an output
/// property every Function has, with the ordinary result position one row south
/// as the default one. A Function carrying this declines that default.
///
/// Scheduling still reads the displacement and the bundle before any Function
/// evaluates, so it can reserve destinations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceEffect {
    /// Cells to displace horizontally, positive to the east.
    pub columns: i16,
    /// Rows to displace vertically, positive to the south.
    pub rows: i16,
    /// The characters written at the displaced Span, or `None` for a bundle
    /// that writes no spelling of its own.
    pub spelling: Option<Spelling>,
    /// Which validated bundle this effect plans.
    pub bundle: SourceBundle,
}

/// Coordinates of one Portal a Function names, relative to its anchor.
///
/// Same seam as [`SourceEffect`]: `lang` answers how far from the producer,
/// and `orcvs` turns that into a Position. A Function that does not name other
/// coordinates uses one row south, `{ columns: 0, rows: 1 }`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PortalCoords {
    /// Cells to displace horizontally, positive to the east.
    pub columns: i16,
    /// Rows to displace vertically, positive to the south.
    pub rows: i16,
}

impl PortalCoords {
    /// One row south of the Function's anchor.
    pub const SOUTH: Self = Self {
        columns: 0,
        rows: 1,
    };
}

/// What a Function does through its Output Portal.
///
/// Every Function does exactly one of these, so there is no variant for
/// none. A value answer is written at a static or a dynamic Output Portal,
/// Halt locks the root through its Output Portal, a Source-writing Function
/// writes its declared [`SourceEffect`] instead, and a Terminal Output
/// Function performs a Play Command and writes no Cell, so its Portal is
/// never addressed and carries no coordinates. A consumer matches on this
/// once rather than asking narrower questions in an order of its own, so a
/// new kind of output fails to compile at every consumer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Output {
    /// The value answer is written at a fixed offset from the anchor.
    Static(PortalCoords),
    /// The value answer is written at the pair the operands select at the
    /// Turn, by the rule the Function's definition names.
    Dynamic(PairSelection),
    /// The declared Source write, resolved against the Grid by `orcvs`.
    Source(SourceEffect),
    /// The Expression root is locked through the Portal at this offset.
    Lock(PortalCoords),
    /// A Play Command delivered to the Playback Engine, with nothing written
    /// back into the Source.
    Terminal,
}

/// Where a Function's Input Portal stands, and when that position is known.
///
/// It is the Portal's position that is static or dynamic, not its contents:
/// every Input Portal's Cells are read from working Source at the Function's
/// Turn. A static Input Portal's position is known when the Source is parsed,
/// so scheduling orders its writers before the Tick. A dynamic Input Portal's
/// position is known only at the Function's Turn, from its operands, so its
/// writers are found then. [`InputPortal::resolve`] answers either kind's
/// [`PortalAddress`] at the Turn, and `orcvs` resolves that address against
/// the Grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputPortal {
    /// A position at a fixed offset from the Function's anchor.
    Static(PortalCoords),
    /// A position the operands select at the Turn, by the rule the
    /// Function's definition names.
    Dynamic(PairSelection),
}

/// Where an Input Portal stands once its Function's operands are resolved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortalAddress {
    /// An offset from the Function's anchor.
    Offset(PortalCoords),
    /// A Position, column then row, counted in Cells from `00 00` at the
    /// Grid's top-left, wherever the Function stands.
    Position {
        /// The Position's column.
        column: u8,
        /// The Position's row.
        row: u8,
    },
}

/// How a dynamic Portal's operands select its Cell pair.
///
/// Each Function with a dynamic Input Portal names its rule in
/// [`Function::input_portal`], and each with a dynamic Output Portal in
/// [`Function::output`]'s [`Output::Dynamic`]. [`PairSelection::resolve`] asks the
/// rule where the pair stands. A rule takes the values of only its address
/// operands: those that lead its Function's signature, except that the
/// absolute Copy's destination rule takes its last two. The directional and
/// absolute rules are declared in the Read's or Write's row, whose signature
/// states how many operands it takes; a Write's `value` follows its address,
/// so a Write and the Read with the same arrow select the same pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairSelection {
    /// The pair `index % count` pairs east of the end of the last operand, on
    /// the Function's own row. Track's selection.
    IndexModuloCount,
    /// The pair `index % count` pairs east of the Function's default Output
    /// Portal: a lane of `count` pairs on the row below the Function,
    /// starting directly under its anchor, wherever its operands end. Push's
    /// selection.
    Lane,
    /// The pair `n` Portals from the `n` operand's slot, the pair immediately
    /// after the Function's spelling, in the direction its [`DistanceRule`]
    /// counts. Each step is one pair east or west, or one row north or south
    /// in the slot's columns, so a distance of zero is the slot itself. A
    /// directional Read's and a directional Write's selection; the rule is
    /// the one its row declares and binds that Function's own operands.
    Distance(DistanceRule),
    /// The pair at the Position its `column` and `row` operands name,
    /// counted in Cells from `00 00` at the Grid's top-left, wherever the
    /// Function stands. The absolute Read's and the absolute Write's
    /// selection; the rule is the one its row declares and binds that
    /// Function's own operands.
    Position(PositionRule),
    /// The pair at the Position the absolute Copy's first two operands,
    /// `src-column` and `src-row`, name: the pair it reads.
    SourcePosition,
    /// The pair at the Position the absolute Copy's last two operands,
    /// `dst-column` and `dst-row`, name: the pair it writes.
    DestinationPosition,
}

/// The direction a directional Read's or Write's arrow names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Toward row zero.
    North,
    /// Away from row zero.
    South,
    /// Away from column zero.
    East,
    /// Toward column zero.
    West,
}

impl Direction {
    /// One Portal in this direction: a pair of Cells east or west, or a row
    /// north or south.
    pub(crate) const fn step(self) -> PortalCoords {
        let pair = expression::DEFAULT_TOKEN_LEN as i16;
        match self {
            Self::North => PortalCoords {
                columns: 0,
                rows: -1,
            },
            Self::South => PortalCoords::SOUTH,
            Self::East => PortalCoords {
                columns: pair,
                rows: 0,
            },
            Self::West => PortalCoords {
                columns: -pair,
                rows: 0,
            },
        }
    }
}

/// Which validated effect bundle a Source-writing Function plans.
///
/// The two Source-writing Function groups differ here and in their activation
/// source, and in nothing else: `*^` and `^^` write the same spelling at the
/// same kind of declared Portal. ADR 0029 refuses to collapse the activation
/// asymmetry, and this is the other half of the same statement — what a
/// producer does with the Cells it is standing in. The Bang Function `**`
/// declares the third: it writes nothing but the clear of its own Span.
///
/// Each variant fixes what a refusal costs as well as what a success writes,
/// because the two are one fact: a bundle that plans the producer's own Span
/// has somewhere to report a refusal, and a bundle that does not has nowhere.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SourceBundle {
    /// Two Portals: spaces over the producer's own Span, then `spelling` at
    /// that Span displaced. The producer vacates the Cells it stood in, so
    /// ADR 0006 has a refused destination replace them with `**` rather than
    /// diagnose — the Span is part of what this bundle plans either way.
    Advance,
    /// One Portal: `spelling` at the displaced Span, with the producer left
    /// standing. ADR 0006 has a refused destination diagnose and emit nothing,
    /// because this bundle plans nothing at the producer's own Cells to say it
    /// with.
    Emit,
    /// One Portal: spaces over the producer's own Span, with Bang delivered
    /// to the roots aligned with that Span. The producer vacates the Cells it
    /// stood in and writes nothing elsewhere, so it can be refused nothing.
    Fire,
}

impl SourceBundle {
    /// Whether the bundle's first write is spaces over the producer's own
    /// Span, so the producer leaves the Cells it stands in: an `Advance` and a
    /// `Fire`.
    pub const fn clears_own_span(self) -> bool {
        match self {
            Self::Advance | Self::Fire => true,
            Self::Emit => false,
        }
    }

    /// Whether the bundle writes its effect's spelling at the producer's Span
    /// displaced: an `Advance` and an `Emit`. Only that write can be refused,
    /// and a bundle that does both moves.
    pub const fn writes_displaced(self) -> bool {
        match self {
            Self::Advance | Self::Emit => true,
            Self::Fire => false,
        }
    }
}

/// The Number two uppercase hexadecimal Cells spell.
///
/// Borrows the spelling and builds nothing on refusal, so a caller that only
/// asks whether two Cells spell a Number allocates nothing to learn that they
/// do not. [`str_to_num`] answers through this and builds its
/// [`TypeError::Number`] only where a refusal is reported.
pub(crate) fn number_from_spelling(s: &str) -> Option<u8> {
    if s.len() != 2
        || !s
            .bytes()
            .all(|cell| cell.is_ascii_digit() || (b'A'..=b'F').contains(&cell))
    {
        return None;
    }

    u8::from_str_radix(s, 16).ok()
}

#[inline(always)]
pub fn str_to_num(s: &str) -> Result<u8, Error> {
    number_from_spelling(s).ok_or_else(|| TypeError::Number(s.to_string()).into())
}

#[cfg(test)]
static INIT: Once = Once::new();

#[cfg(test)]
fn trace() {
    INIT.call_once(|| {
        use tracing_subscriber::FmtSubscriber;

        let subscriber = FmtSubscriber::builder()
            .with_max_level(tracing::Level::DEBUG) // Set the maximum level of tracing events that should be logged.
            .with_line_number(true)
            .with_target(true)
            // No timestamps: Miri's isolation rejects the wall-clock read one makes.
            .without_time()
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    });
}

/// The pitch letters a Note spells, in semitone order from C.
pub(crate) const NOTE_PITCHES: &[u8; 12] = b"CcDdEFfGgAaB";

/// The Cell that spells a Note's octave: `/` for the lowest, then the digits
/// `0` to `9`. [`octave_of_cell`] reads it back.
pub(crate) const fn octave_cell(octave: u8) -> u8 {
    match octave {
        0 => b'/',
        octave => b'0' + octave - 1,
    }
}

/// The octave a Note's octave Cell spells, the inverse of [`octave_cell`].
fn octave_of_cell(cell: u8) -> Option<u8> {
    match cell {
        b'/' => Some(0),
        b'0'..=b'9' => Some(cell - b'0' + 1),
        _ => None,
    }
}

pub fn midi_note_to_number(note: &str) -> Option<u8> {
    let [pitch, octave] = *note.as_bytes() else {
        return None;
    };
    let pitch = u8::try_from(
        NOTE_PITCHES
            .iter()
            .position(|candidate| *candidate == pitch)?,
    )
    .ok()?;
    let octave = octave_of_cell(octave)?;

    octave
        .checked_mul(12)?
        .checked_add(pitch)
        .filter(|note| *note <= 0x7F)
}

/// Evaluates Source text spelling one Function over Operand Literals, the way
/// a Turn calls the Interpreter once it has resolved the operands: the Parser
/// reads the spelling and [`Interpreter::execute_function`] answers it.
/// Nesting is resolved by `orcvs`, so a nested Function is refused here rather
/// than evaluated by a second path.
#[cfg(test)]
fn interpret_source(source: &str) -> Result<Interpretation, Error> {
    let atoms = Parser::from(source).try_parse()?;
    let Some((Atom::Function(function), literals)) = atoms.split_first() else {
        panic!("{source:?} does not start with a Function");
    };
    let operands: Vec<Atom> = literals
        .iter()
        .map(|literal| match literal {
            Atom::Function(nested) => panic!("{source:?} nests {nested}; a Turn resolves it first"),
            literal => *literal,
        })
        .collect();
    Interpreter::execute_function(
        *function,
        operands,
        TickInputs::new(Tick::ZERO, Anchor::new(0, 0)).into(),
    )
}

#[cfg(test)]
mod test {
    use super::{Atom, Note, midi_note_to_number, str_to_num};

    #[test]
    fn the_answer_seam_is_the_size_the_execute_function_benchmark_was_measured_against() {
        use super::{Interpretation, PlayCommand};

        // A layout claim, pinned because the `execute_function` benchmark
        // measures the answer seam. The widest answer is a Source effect: two
        // `i16` displacements, a three-byte `Option<Spelling>` and its bundle.
        // No field holds a pointer or `usize`, so `Interpretation` reads 8 at
        // every pointer width, wasm32 included.
        //
        // A failure here is notice rather than a defect: the answer seam has
        // changed shape, and `execute_function` is the measurement to take
        // again.
        assert_eq!(size_of::<PlayCommand>(), 5);
        assert_eq!(size_of::<Interpretation>(), 8);
    }

    #[test]
    fn test_str_to_num_rejects_a_leading_sign() {
        // `u8::from_str_radix` accepts a leading `+`, which would let a stray
        // `+` prepended to an Expression parse as a valid operand instead of
        // failing: `+++0101` would read its first operand as `+0` == 0
        assert!(str_to_num("+0").is_err());
        assert!(str_to_num("+F").is_err());
        assert!(str_to_num("-1").is_err());
    }

    #[test]
    fn every_number_has_one_canonical_two_cell_source_encoding() {
        for number in 0..=u8::MAX {
            let source = format!("{number:02X}");
            assert_eq!(str_to_num(&source).unwrap(), number);
            assert_eq!(Atom::Number(number).to_string(), source);
        }
    }

    #[test]
    fn non_canonical_number_source_encodings_diagnose() {
        for source in ["", "0", "A", "abc", "0a", "+0", "-1", "000", "FFF"] {
            assert!(str_to_num(source).is_err(), "accepted {source:?}");
        }
    }

    /// Every two-character spelling an ASCII Source can write.
    ///
    /// A Number and a Note each occupy two Cells, and a Cell holds one ASCII
    /// character, so this is the complete candidate set for either reading
    /// rather than a sample of it. Enumerating it is what lets the two sweeps
    /// below say which spellings are accepted, instead of only that some named
    /// rejected ones are rejected.
    fn two_cell_ascii_spellings() -> impl Iterator<Item = String> {
        (0u8..=0x7F).flat_map(|first| {
            (0u8..=0x7F)
                .map(move |second| String::from_utf8(vec![first, second]).expect("ASCII is UTF-8"))
        })
    }

    /// The value of one uppercase hexadecimal Cell.
    ///
    /// Written out here rather than delegated to `from_str_radix`, which is
    /// the thing under test and accepts spellings this does not.
    fn hex_digit(cell: u8) -> Option<u8> {
        let index = "0123456789ABCDEF".bytes().position(|digit| digit == cell)?;
        u8::try_from(index).ok()
    }

    #[test]
    fn str_to_num_accepts_exactly_the_uppercase_two_cell_hexadecimal_spellings() {
        // A Number renders back into the Source in uppercase and is re-read
        // from it on the next Tick, so one value must have exactly one
        // spelling and one spelling exactly one value. Sweeping the candidates
        // is what makes this an "exactly": the lowercase spelling of every
        // value is refused by the sweep rather than by the sampled list above.
        let mut accepted = 0usize;

        for source in two_cell_ascii_spellings() {
            let [high, low] = *source.as_bytes() else {
                unreachable!("two ASCII Cells");
            };

            match (hex_digit(high), hex_digit(low)) {
                (Some(high), Some(low)) => {
                    assert_eq!(str_to_num(&source).unwrap(), high * 16 + low, "{source:?}");
                    accepted += 1;
                }
                _ => assert!(str_to_num(&source).is_err(), "accepted {source:?}"),
            }
        }

        // One accepted spelling per Number and no more.
        assert_eq!(accepted, 256);
    }

    #[test]
    fn midi_note_to_number_accepts_exactly_the_pitch_spellings_inside_the_midi_range() {
        // The counterpart sweep, and the half of the range claim a round trip
        // cannot make: `A9` and `B9` are spelled the way every other Note is
        // and name 129 and 131, so a reading that computed a pitch without
        // checking the range would accept them and mint a Note with no MIDI
        // byte behind it.
        let mut accepted = 0usize;

        for source in two_cell_ascii_spellings() {
            let [pitch, octave] = *source.as_bytes() else {
                unreachable!("two ASCII Cells");
            };

            // The Note spelling, stated here rather than read from the
            // table under test: twelve chromatic pitches to the octave, from
            // `C/` through `G9`.
            let pitch = "CcDdEFfGgAaB"
                .bytes()
                .position(|candidate| candidate == pitch)
                .and_then(|pitch| u16::try_from(pitch).ok());
            let octave = match octave {
                b'/' => Some(0u16),
                b'0'..=b'9' => Some(u16::from(octave - b'0') + 1),
                _ => None,
            };
            let expected = match (pitch, octave) {
                (Some(pitch), Some(octave)) => u8::try_from(octave * 12 + pitch)
                    .ok()
                    .filter(|n| *n <= 0x7F),
                _ => None,
            };

            assert_eq!(midi_note_to_number(&source), expected, "{source:?}");
            if expected.is_some() {
                accepted += 1;
            }
        }

        // One accepted spelling per MIDI value and no more.
        assert_eq!(accepted, 128);
    }

    #[test]
    fn every_midi_note_round_trips_through_its_two_cell_source_encoding() {
        for number in 0x00..=0x7F {
            let source = Atom::Note(Note::try_from(number).unwrap()).to_string();
            assert_eq!(source.len(), 2, "Note({number}) rendered as {source:?}");
            assert_eq!(midi_note_to_number(&source), Some(number));
        }
    }

    #[test]
    fn note_source_encoding_covers_the_documented_boundaries() {
        assert_eq!(Atom::Note(Note::try_from(0).unwrap()).to_string(), "C/");
        assert_eq!(midi_note_to_number("C/"), Some(0));
        assert_eq!(Atom::Note(Note::try_from(127).unwrap()).to_string(), "G9");
        assert_eq!(midi_note_to_number("G9"), Some(127));
    }

    #[test]
    fn values_above_the_midi_range_have_no_note_source_encoding() {
        for number in 0x80..=u8::MAX {
            assert!(Note::try_from(number).is_err(), "Note({number}) exists");
        }

        for source in ["g9", "H4", "C:", "C10", ""] {
            assert_eq!(midi_note_to_number(source), None, "accepted {source:?}");
        }
    }
}
