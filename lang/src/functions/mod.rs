pub(crate) mod math;
pub(crate) mod numeric_conversion;
use crate::{Error, Function, InterpretationError, PlayCommand, interpreter::Context};

/// The MIDI channel domain, shared by every Terminal Output Function.
///
/// Orcvs sends direct hexadecimal MIDI values, so an operand outside the
/// protocol range is a Source error rather than something to scale or clamp.
/// Validation lives here instead of in each Function so `!c` and `!b` inherit
/// the same domain and the same diagnostic wording as `!>`.
#[inline(always)]
pub(crate) fn midi_channel(channel: u8) -> Result<u8, Error> {
    match channel {
        0x00..=0x0F => Ok(channel),
        _ => Err(InterpretationError::MidiChannel(channel).into()),
    }
}

/// The MIDI data-byte domain, named by the operand role it is checking.
///
/// A Play velocity, a Control Change controller or value, and a Pitch Bend LSB
/// or MSB all occupy `00`–`7F`; only the word the diagnostic uses differs, so
/// `role` is the whole difference between them.
#[inline(always)]
pub(crate) fn midi_data_byte(role: &'static str, value: u8) -> Result<u8, Error> {
    match value {
        0x00..=0x7F => Ok(value),
        _ => Err(InterpretationError::MidiDataByte { role, value }.into()),
    }
}

/// Raw Play: `!> channel velocity note`.
///
/// Every operand is checked against its MIDI domain before any Play Command
/// exists, so a Function that diagnoses emits nothing at all. The Note operand
/// needs no check here: `Stack::extract` accepts only a `Note`, whose own
/// construction already enforces `00`–`7F`.
#[inline(always)]
pub fn play(ctx: &mut Context) -> Result<PlayCommand, Error> {
    let operands = ctx.stack.extract(Function::Play)?;

    let channel = midi_channel(operands.number(0))?;
    let velocity = midi_data_byte("velocity", operands.number(1))?;
    let note = operands.note(2).value();

    Ok(PlayCommand::Raw {
        channel,
        velocity,
        note,
    })
}

#[cfg(test)]
mod test {
    use super::{midi_channel, midi_data_byte, play};
    use crate::{
        ArgumentError, Atom, Error, InterpretationError, PlayCommand, interpreter::Context,
    };

    /// Pins the Play arity contract before issue 04 replaces the placeholder.
    /// See `.scratch/source-playback-engine/issues/04-interpret-terminal-play-functions-into-play-commands.md`
    #[test]
    fn test_play_consumes_exactly_three_arguments() {
        let mut ctx = Context::new();

        // A fourth atom below the three arguments must survive untouched
        ctx.stack.push(Atom::Char('z'));
        ctx.stack
            .push(Atom::Note(crate::Note::try_from(60).unwrap())); // n
        ctx.stack.push(Atom::Number(0x7F)); // v
        ctx.stack.push(Atom::Number(0x0)); // c

        let result = play(&mut ctx).unwrap();

        assert_eq!(
            result,
            PlayCommand::Raw {
                channel: 0,
                velocity: 0x7F,
                note: 60,
            }
        );

        // Exactly three arguments were consumed
        assert_eq!(Atom::from(ctx.stack.pop().unwrap()), Atom::Char('z'));
        assert_eq!(Atom::from(ctx.stack.pop().unwrap()), Atom::Empty);
    }

    #[test]
    fn test_play_requires_three_arguments() {
        for found in 0..3 {
            let mut ctx = Context::new();
            for _ in 0..found {
                ctx.stack.push(Atom::Number(1));
            }

            let error = play(&mut ctx).unwrap_err();

            assert!(
                matches!(
                    error,
                    Error::Argument(ArgumentError::Arity { expected: 3, found: f }) if f == found
                ),
                "{found} argument(s) gave {error:?}"
            );
        }
    }

    #[test]
    fn play_rejects_implicit_number_note_conversions() {
        for arguments in [
            [
                Atom::Note(crate::Note::try_from(0).unwrap()),
                Atom::Number(0x7F),
                Atom::Note(crate::Note::try_from(60).unwrap()),
            ],
            [
                Atom::Number(0),
                Atom::Note(crate::Note::try_from(0x7F).unwrap()),
                Atom::Note(crate::Note::try_from(60).unwrap()),
            ],
            [Atom::Number(0), Atom::Number(0x7F), Atom::Number(60)],
        ] {
            let mut ctx = Context::new();
            for argument in arguments.into_iter().rev() {
                ctx.stack.push(argument);
            }

            assert!(matches!(play(&mut ctx), Err(Error::Type(_))));
        }
    }

    #[test]
    fn play_rejects_channels_outside_the_midi_range() {
        for channel in 0x10..=u8::MAX {
            let mut ctx = Context::new();
            for argument in [
                Atom::Number(channel),
                Atom::Number(0x7F),
                Atom::Note(crate::Note::try_from(60).unwrap()),
            ]
            .into_iter()
            .rev()
            {
                ctx.stack.push(argument);
            }

            assert!(matches!(
                play(&mut ctx),
                Err(Error::Interpretation(InterpretationError::MidiChannel(value)))
                    if value == channel
            ));
        }
    }

    #[test]
    fn play_rejects_velocities_outside_the_midi_data_byte_range() {
        for velocity in 0x80..=u8::MAX {
            let mut ctx = Context::new();
            for argument in [
                Atom::Number(0),
                Atom::Number(velocity),
                Atom::Note(crate::Note::try_from(60).unwrap()),
            ]
            .into_iter()
            .rev()
            {
                ctx.stack.push(argument);
            }

            assert!(matches!(
                play(&mut ctx),
                Err(Error::Interpretation(InterpretationError::MidiDataByte {
                    role: "velocity",
                    value,
                })) if value == velocity
            ));
        }
    }

    #[test]
    fn the_shared_midi_domains_accept_exactly_their_protocol_ranges() {
        for value in 0..=u8::MAX {
            assert_eq!(midi_channel(value).is_ok(), value <= 0x0F, "{value:02X}");
            assert_eq!(
                midi_data_byte("velocity", value).is_ok(),
                value <= 0x7F,
                "{value:02X}"
            );
        }
    }

    #[test]
    fn a_rejected_data_byte_names_the_operand_role_that_supplied_it() {
        // Issues 02-04 reuse this seam unchanged: only the role word differs
        // between a Play velocity, a Control Change value, and a Pitch Bend MSB.
        for role in ["velocity", "controller", "value", "lsb", "msb"] {
            let error = midi_data_byte(role, 0x80).unwrap_err();

            assert_eq!(
                error.to_string(),
                format!("MIDI {role} 80 is outside the range 00–7F")
            );
        }

        assert_eq!(
            midi_channel(0x10).unwrap_err().to_string(),
            "MIDI channel 10 is outside the range 00–0F"
        );
    }
}
