mod atom;
mod error;
mod expression;
mod functions;
mod interpreter;
mod parser;
mod portal;
mod stack;

pub use atom::{Activation, Atom, Atoms, Function, Note, to_atom_note, to_atom_num};
pub use error::{ArgumentError, Error, InterpretationError, SyntaxError, TypeError};
pub use expression::{Expression, Token, Tokens};
pub use interpreter::{Interpretation, Interpreter};
pub use parser::Parser;
pub use portal::Portal;
pub use stack::Stack;

use std::sync::Once;

pub const EXP_LEN: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayCommand {
    pub channel: u8,
    pub velocity: u8,
    pub note: u8,
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
    use super::{Atom, Note, midi_note_to_number, midi_number_to_note, str_to_num};

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
