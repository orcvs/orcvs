pub(crate) mod math;
pub(crate) mod numeric_conversion;
use crate::{Error, PlayCommand, atom::operands, interpreter::Context};

/// Raw Play: `!> channel velocity note`.
///
/// There is no validation call here. Each operand's domain is declared beside
/// its role in `define_functions!` and converted during extraction, so a
/// Function that diagnoses has produced no Play Command at all, and a new MIDI
/// terminal Function inherits its validation from its declaration rather than
/// from a body that remembers to ask for it.
#[inline(always)]
pub fn play(ctx: &mut Context) -> Result<PlayCommand, Error> {
    let operands::Play {
        channel,
        velocity,
        note,
    } = ctx.stack.extract()?;

    Ok(PlayCommand::Raw {
        channel,
        velocity,
        note,
    })
}

#[cfg(test)]
mod test {
    use super::play;
    use crate::{
        Anchor, ArgumentError, Atom, Error, Interpretation, InterpretationError, Interpreter,
        MidiChannel, Note, Parser, PlayCommand, Tick, TickInputs, Velocity, interpreter::Context,
    };

    ///
    /// The Tick inputs for a test about operands rather than about time or
    /// Position: the first Tick of a Playback run, at the Grid origin.
    ///
    fn inputs() -> TickInputs {
        TickInputs::new(Tick::ZERO, Anchor::new(0, 0))
    }

    ///
    /// A Context for a test about operands rather than about time or Position.
    ///
    fn context() -> Context {
        Context::new(inputs())
    }

    /// Evaluates `source` as one Expression, exactly as a Tick would.
    fn interpret(source: &str) -> Result<Interpretation, Error> {
        let mut source = source.to_string();
        let atoms = Parser::from(&mut source).try_parse().unwrap();
        Interpreter::execute(&atoms, inputs())
    }

    /// Pins the Play arity contract before issue 04 replaces the placeholder.
    /// See `.scratch/source-playback-engine/issues/04-interpret-terminal-play-functions-into-play-commands.md`
    #[test]
    fn test_play_consumes_exactly_three_arguments() {
        let mut ctx = context();

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
                channel: MidiChannel::try_from(0).unwrap(),
                velocity: Velocity::try_from(0x7F).unwrap(),
                note: Note::try_from(60).unwrap(),
            }
        );

        // Exactly three arguments were consumed
        assert_eq!(Atom::from(ctx.stack.pop().unwrap()), Atom::Char('z'));
        assert_eq!(Atom::from(ctx.stack.pop().unwrap()), Atom::Empty);
    }

    #[test]
    fn play_carries_each_operand_into_the_role_its_signature_names() {
        // 01 and 02 are legal as a channel and as a data byte, and a complete
        // role swap inside the declaration carries each name with its own type,
        // so it still compiles and every operand still lands in a legal domain.
        // Only differing operand values separate a correct declaration from a
        // transposed one, which is why this test cannot be replaced by the
        // domain types it sits beside.
        let mut ctx = context();
        ctx.stack
            .push(Atom::Note(crate::Note::try_from(60).unwrap()));
        ctx.stack.push(Atom::Number(0x02));
        ctx.stack.push(Atom::Number(0x01));

        let expected = PlayCommand::Raw {
            channel: MidiChannel::try_from(0x01).unwrap(),
            velocity: Velocity::try_from(0x02).unwrap(),
            note: Note::try_from(60).unwrap(),
        };

        assert_eq!(play(&mut ctx).unwrap(), expected);

        // The same claim from Source text, which adds the parse and the
        // right-to-left walk to what the extraction alone proves: `!>` reads
        // channel, then velocity, then note, left to right in the Cells.
        assert_eq!(
            interpret("!>0102C4").unwrap(),
            Interpretation::Play(expected)
        );
    }

    #[test]
    fn test_play_requires_three_arguments() {
        for found in 0..3 {
            let mut ctx = context();
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
            let mut ctx = context();
            for argument in arguments.into_iter().rev() {
                ctx.stack.push(argument);
            }

            assert!(matches!(play(&mut ctx), Err(Error::Type(_))));
        }
    }

    #[test]
    fn play_rejects_channels_outside_the_midi_range() {
        for channel in 0x10..=u8::MAX {
            let mut ctx = context();
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
            let mut ctx = context();
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
    fn a_non_numeric_value_diagnoses_where_a_numeric_conversion_consumes_it() {
        // `TypeError::Numeric` is reachable from Source: `.=` answers a Bang
        // for equal operands, and `.^` pops it expecting a Number or a Note.
        // The stack seam is unit-tested where it lives; this is the Source
        // spelling that proves it is reachable at all.
        assert!(matches!(
            interpret(".^.=0101"),
            Err(Error::Type(crate::TypeError::Numeric(found))) if found == "**"
        ));
    }

    #[test]
    fn an_out_of_domain_operand_produces_no_play_command_at_all() {
        // The domain conversion happens during extraction, so a Play that
        // diagnoses has never constructed a PlayCommand to be discarded.
        assert!(matches!(
            interpret("!>107FC4"),
            Err(Error::Interpretation(InterpretationError::MidiChannel(
                0x10
            )))
        ));
    }
}
