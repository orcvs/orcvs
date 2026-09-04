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
pub fn raw_play(ctx: &mut Context) -> Result<PlayCommand, Error> {
    let operands::RawPlay {
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

/// Timed Play: `!~ channel velocity note length`.
///
/// The length is validated as the Number it is and carried on unread. ADR 0016
/// requires it even where it changes nothing — velocity `00` stops the note
/// whatever length accompanies it — so the operand is extracted here and what
/// it means is decided by the Playback Engine that owns the Ticks it counts.
#[inline(always)]
pub fn timed_play(ctx: &mut Context) -> Result<PlayCommand, Error> {
    let operands::TimedPlay {
        channel,
        velocity,
        note,
        length,
    } = ctx.stack.extract()?;

    Ok(PlayCommand::Timed {
        channel,
        velocity,
        note,
        length,
    })
}

#[cfg(test)]
mod test {
    use super::{raw_play, timed_play};
    use crate::{
        Anchor, ArgumentError, Atom, Error, Interpretation, InterpretationError, Interpreter,
        Length, MidiChannel, Note, Parser, PlayCommand, Tick, TickInputs, Velocity,
        interpreter::Context,
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
    fn test_raw_play_consumes_exactly_three_arguments() {
        let mut ctx = context();

        // A fourth atom below the three arguments must survive untouched
        ctx.stack.push(Atom::Char('z'));
        ctx.stack
            .push(Atom::Note(crate::Note::try_from(60).unwrap())); // n
        ctx.stack.push(Atom::Number(0x7F)); // v
        ctx.stack.push(Atom::Number(0x0)); // c

        let result = raw_play(&mut ctx).unwrap();

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

        assert_eq!(raw_play(&mut ctx).unwrap(), expected);

        // The same claim from Source text, which adds the parse and the
        // right-to-left walk to what the extraction alone proves: `!>` reads
        // channel, then velocity, then note, left to right in the Cells.
        assert_eq!(
            interpret("!>0102C4").unwrap(),
            Interpretation::Play(expected)
        );
    }

    #[test]
    fn timed_play_carries_each_operand_into_the_role_its_signature_names() {
        // Four operands and four differing values, for the reason Raw Play's
        // role test gives: a complete transposition inside the declaration
        // carries each name with its own type and compiles, so only values
        // that differ from one another separate it from the declaration meant.
        // Channel and length are the exposed pair here — `01` and `04` are
        // legal in both domains — which is why they are the two furthest apart.
        let mut ctx = context();
        ctx.stack.push(Atom::Number(0x04));
        ctx.stack
            .push(Atom::Note(crate::Note::try_from(60).unwrap()));
        ctx.stack.push(Atom::Number(0x02));
        ctx.stack.push(Atom::Number(0x01));

        let expected = PlayCommand::Timed {
            channel: MidiChannel::try_from(0x01).unwrap(),
            velocity: Velocity::try_from(0x02).unwrap(),
            note: Note::try_from(60).unwrap(),
            length: Length::from(0x04),
        };

        assert_eq!(timed_play(&mut ctx).unwrap(), expected);

        // And the same claim from Source text, which adds the parse and the
        // right-to-left walk: `!~` reads channel, velocity, note, then length,
        // left to right in the Cells.
        assert_eq!(
            interpret("!~0102C404").unwrap(),
            Interpretation::Play(expected)
        );
    }

    #[test]
    fn timed_play_requires_four_arguments() {
        // Each prefix of a well-typed operand list, so what is missing is the
        // count rather than a type: an arity diagnostic must precede every
        // other one, and only a correctly typed prefix can prove it does.
        let operands = [
            Atom::Number(0x01),
            Atom::Number(0x02),
            Atom::Note(crate::Note::try_from(60).unwrap()),
            Atom::Number(0x04),
        ];

        for found in 0..4 {
            let mut ctx = context();
            for argument in operands.iter().take(found).rev() {
                ctx.stack.push(*argument);
            }

            let error = timed_play(&mut ctx).unwrap_err();

            assert!(
                matches!(
                    error,
                    Error::Argument(ArgumentError::Arity { expected: 4, found: f }) if f == found
                ),
                "{found} argument(s) gave {error:?}"
            );
        }
    }

    #[test]
    fn timed_play_takes_the_same_midi_domains_as_raw_play_and_a_whole_byte_of_length() {
        // The domains ADR 0016 fixes for `!~`, each proven by the operand that
        // leaves them: a length is the one operand with nothing outside it.
        assert!(matches!(
            interpret("!~107FC401"),
            Err(Error::Interpretation(InterpretationError::MidiChannel(
                0x10
            )))
        ));
        assert!(matches!(
            interpret("!~0080C401"),
            Err(Error::Interpretation(InterpretationError::MidiDataByte {
                role: "velocity",
                value: 0x80,
            }))
        ));
        // A Number in the note position is refused at evaluation as well as at
        // the parse: `.v` answers one from Source text, and Play infers no Note.
        assert!(matches!(
            interpret("!~007F.vC401"),
            Err(Error::Type(crate::TypeError::Note(found))) if found == "3C"
        ));

        for length in 0..=u8::MAX {
            assert_eq!(
                interpret(&format!("!~007FC4{length:02X}")).unwrap(),
                Interpretation::Play(PlayCommand::Timed {
                    channel: MidiChannel::try_from(0).unwrap(),
                    velocity: Velocity::try_from(0x7F).unwrap(),
                    note: Note::try_from(60).unwrap(),
                    length: Length::from(length),
                }),
                "length {length:02X}"
            );
        }
    }

    #[test]
    fn test_raw_play_requires_three_arguments() {
        for found in 0..3 {
            let mut ctx = context();
            for _ in 0..found {
                ctx.stack.push(Atom::Number(1));
            }

            let error = raw_play(&mut ctx).unwrap_err();

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

            assert!(matches!(raw_play(&mut ctx), Err(Error::Type(_))));
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
                raw_play(&mut ctx),
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
                raw_play(&mut ctx),
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
