mod atom;
mod error;
mod expression;
mod functions;
mod interpreter;
mod parser;
mod sequence;
mod stack;
mod tick;

pub use atom::{
    Activation, Atom, Atoms, Function, Length, MidiChannel, Note, Velocity, to_atom_note,
    to_atom_num,
};
pub use error::{ArgumentError, Error, InterpretationError, SequenceError, SyntaxError, TypeError};
pub use expression::{Expression, Token, Tokens};
pub use interpreter::{Interpretation, Interpreter};
pub use parser::{Parser, SourceAnalysis};
pub use sequence::{Sequence, Value};
pub use stack::Stack;
pub use tick::{Anchor, Tick, TickInputs};

use std::sync::Once;

pub const EXP_LEN: usize = 32;

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
    /// ADR 0016's Raw Play. Velocity `00` is not an absent note but the
    /// explicit stop MIDI's zero-velocity convention gives the Source.
    Raw {
        channel: MidiChannel,
        velocity: Velocity,
        note: Note,
    },
    ///
    /// ADR 0016's Timed Play, carrying the whole lifetime the Source wrote.
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
}

/// The ordered group of Play Commands one Terminal Output Function Expression
/// performs.
///
/// ADR 0030 extends the Terminal Output Functions pervasively over a Sequence
/// operand, so one Expression can perform many times while still answering no
/// value: ADR 0028 bounds the kind of answer an instruction gives, not how much
/// of it, and a Play Command is never encoded into Cells, so the rules that
/// make a Sequence expensive where a result becomes Source do not reach an
/// effect. Order within the group is element index, which ADR 0030 chooses
/// because it is the only order the Source can read — the order the Cells would
/// have if the same notes were written left to right as separate Expressions.
///
/// Two shapes rather than one, the way [`Value`] keeps `Atom` beside
/// `Sequence`. Every Play a Source has written so far is scalar, and answering
/// a group of one for it would put a heap allocation on the path that has none.
/// [`Performance::Many`] is legitimately empty: an empty Sequence operand is a
/// real width of no elements, and an Expression of no elements performs no MIDI
/// output rather than diagnosing.
///
/// Equality compares shapes, so `One(command)` and a `Many` holding that same
/// one command are unequal even though [`Performance::commands`] reads them
/// identically. That is sound because the shape is a fact about the Expression
/// rather than an incidental choice of representation: `Stack::perform` answers
/// `One` for an operation of Atoms alone and `Many` for one a Sequence operand
/// widened, so comparing shapes is comparing values. The two are therefore not
/// distinguished by count. A one-element Sequence operand widens an operation
/// to width one and answers `Many` holding a single command, which is right —
/// a Sequence of one is not an Atom, and the Expression that spelled it is not
/// the Expression that spelled a scalar. What `One` says is that the operands
/// were scalar, not that there is exactly one command.
///
/// The derive is deliberately not flattened to compare `commands()`, because
/// that is what lets a test state that a scalar Play answers one command and
/// not a group of one; a flattened equality would accept both.
#[derive(Clone, Debug, PartialEq)]
pub enum Performance {
    /// A scalar Expression, whose one command is answered without a group
    /// around it.
    One(PlayCommand),
    /// A widened Expression, whose commands are ordered by element index.
    Many(Vec<PlayCommand>),
}

impl Performance {
    /// The commands in order.
    ///
    /// One shape reading, so a consumer delivering a Performance never learns
    /// which of the two it was handed: `Playback` dispatches a Tick Plan's
    /// commands as one list, and the distinction ADR 0026 will revisit is about
    /// what evaluation costs rather than about what delivery sees.
    #[inline(always)]
    pub fn commands(&self) -> &[PlayCommand] {
        match self {
            Self::One(command) => std::slice::from_ref(command),
            Self::Many(commands) => commands,
        }
    }
}

impl<'a> IntoIterator for &'a Performance {
    type Item = &'a PlayCommand;
    type IntoIter = std::slice::Iter<'a, PlayCommand>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.commands().iter()
    }
}

#[inline(always)]
pub fn str_to_num(s: &str) -> Result<u8, Error> {
    if s.len() != 2
        || !s
            .bytes()
            .all(|cell| cell.is_ascii_digit() || (b'A'..=b'F').contains(&cell))
    {
        return Err(TypeError::Number(s.to_string()).into());
    }

    match u8::from_str_radix(s, 16) {
        Ok(n) => Ok(n),
        Err(_) => Err(TypeError::Number(s.to_string()).into()),
    }
}

#[allow(dead_code)]
static INIT: Once = Once::new();

#[allow(dead_code)]
fn trace() {
    INIT.call_once(|| {
        use tracing_subscriber::FmtSubscriber;

        let subscriber = FmtSubscriber::builder()
            .with_max_level(tracing::Level::DEBUG) // Set the maximum level of tracing events that should be logged.
            .with_line_number(true)
            .with_target(true)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    });
}

const NOTE_PITCHES: &[u8; 12] = b"CcDdEFfGgAaB";

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
    let octave = match octave {
        b'/' => 0,
        b'0'..=b'9' => octave - b'0' + 1,
        _ => return None,
    };

    octave
        .checked_mul(12)?
        .checked_add(pitch)
        .filter(|note| *note <= 0x7F)
}

fn midi_number_to_note(note: u8) -> Option<String> {
    if note > 0x7F {
        return None;
    }

    let pitch = char::from(NOTE_PITCHES[usize::from(note % 12)]);
    let octave = match note / 12 {
        0 => '/',
        octave => char::from(b'0' + octave - 1),
    };
    Some(format!("{pitch}{octave}"))
}

#[cfg(test)]
mod test {
    use super::{
        Atom, Interpretation, MidiChannel, Note, Performance, PlayCommand, Velocity,
        midi_note_to_number, midi_number_to_note, str_to_num,
    };

    fn raw(note: u8) -> PlayCommand {
        PlayCommand::Raw {
            channel: MidiChannel::try_from(0).unwrap(),
            velocity: Velocity::try_from(0x7F).unwrap(),
            note: Note::try_from(note).unwrap(),
        }
    }

    #[test]
    fn both_performance_shapes_read_back_as_one_ordered_list_of_commands() {
        // The two shapes exist for what evaluation costs, not for what delivery
        // sees: Playback dispatches a Tick Plan's commands as one list, so a
        // consumer must never have to ask which shape it was handed. A scalar
        // Expression reads back as exactly one command — not as a group of one —
        // and a widened one reads back in element index order.
        assert_eq!(Performance::One(raw(60)).commands(), &[raw(60)]);
        assert_eq!(
            Performance::Many(vec![raw(60), raw(64), raw(67)]).commands(),
            &[raw(60), raw(64), raw(67)]
        );

        // Iteration is the accessor's order, so the seam that flattens a group
        // into a Tick Plan cannot reorder it.
        assert_eq!(
            Performance::Many(vec![raw(60), raw(64)])
                .into_iter()
                .copied()
                .collect::<Vec<_>>(),
            vec![raw(60), raw(64)]
        );

        // An empty group is a legitimate answer rather than an absent one: an
        // empty Sequence operand is a real width of no elements.
        assert!(Performance::Many(Vec::new()).commands().is_empty());

        // The shapes are not interchangeable, which is what the derived
        // equality is for: a scalar Expression answers `One`, and a test that
        // says so must be able to fail when a group of one is answered instead.
        assert_ne!(Performance::One(raw(60)), Performance::Many(vec![raw(60)]));
    }

    #[test]
    // The figures are pointer-width dependent, and the prose below explains
    // them in terms of a 64-bit niche. Declaring that to the compiler rather
    // than to the reader is what keeps a 32-bit target reporting no defect
    // instead of a size it was never measured at.
    #[cfg(target_pointer_width = "64")]
    fn the_answer_seam_is_the_size_the_execute_benchmark_was_measured_against() {
        // A layout claim, pinned because a benchmark explanation rests on it.
        // `Interpretation` was 24 bytes before this type existed: its widest
        // variant held a `Sequence`, which is a `Vec`, and the non-null pointer
        // left a niche the discriminant fitted into. `Performance` is itself a
        // 24-byte enum that has already spent that niche on its own tag, so
        // `Interpretation` needs one of its own and reads 32. That eight bytes
        // and the discriminant read beside it are what moved the `execute`
        // benchmark from 46 ns to 52 ns — a representation change rather than
        // work added, and the only measured cost of extending the Terminal
        // Output Functions.
        //
        // A failure here is notice rather than a defect: the answer seam has
        // changed shape, and `execute` is the measurement to take again. That
        // is only worth being told where the figures mean something, which is
        // what the `target_pointer_width` gate above says — `wasm32` builds the
        // library and runs its regressions in the `shell` crate, so today the
        // gate excludes nothing that runs.
        assert_eq!(size_of::<Performance>(), 24);
        assert_eq!(size_of::<Interpretation>(), 32);
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

    #[test]
    fn every_midi_note_round_trips_through_its_two_cell_source_encoding() {
        for number in 0x00..=0x7F {
            let source = midi_number_to_note(number).unwrap();
            assert_eq!(source.len(), 2, "Note({number}) rendered as {source:?}");
            assert_eq!(midi_note_to_number(&source), Some(number));
            assert_eq!(
                Atom::Note(Note::try_from(number).unwrap()).to_string(),
                source
            );
        }
    }

    #[test]
    fn note_source_encoding_covers_the_documented_boundaries() {
        assert_eq!(midi_number_to_note(0).as_deref(), Some("C/"));
        assert_eq!(midi_note_to_number("C/"), Some(0));
        assert_eq!(midi_number_to_note(127).as_deref(), Some("G9"));
        assert_eq!(midi_note_to_number("G9"), Some(127));
    }

    #[test]
    fn values_above_the_midi_range_have_no_note_source_encoding() {
        for number in 0x80..=u8::MAX {
            assert_eq!(midi_number_to_note(number), None);
        }

        for source in ["g9", "H4", "C:", "C10", ""] {
            assert_eq!(midi_note_to_number(source), None, "accepted {source:?}");
        }
    }
}
