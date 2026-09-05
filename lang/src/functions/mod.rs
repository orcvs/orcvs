pub(crate) mod math;
pub(crate) mod numeric_conversion;
use crate::{Error, Performance, PlayCommand, atom::operands, interpreter::Context};

// Both Functions here are declared Pervasive in `define_functions!`, so each
// body states one Play Command for one element and says nothing about
// Sequences, exactly as an Atomic Function body states one Atom. ADR 0030
// extends them under ADR 0007's rules rather than under rules of their own:
// `Stack::perform` decides the one shape the operands make, hands out each
// element's operands, and answers the ordered group. A body that walked a
// Sequence itself would be a second broadcast mechanism, free to disagree with
// the first about lengths, about ordering, and about what a partial failure
// leaves sounding.

/// Raw Play: `!> channel velocity note`.
///
/// There is no validation call here. Each operand's domain is declared beside
/// its role in `define_functions!` and converted as each element binds, so a
/// Function that diagnoses has produced no Play Command at all — at any width,
/// because nothing is answered until every element has bound — and a new MIDI
/// terminal Function inherits its validation from its declaration rather than
/// from a body that remembers to ask for it.
#[inline(always)]
pub fn raw_play(ctx: &mut Context) -> Result<Performance, Error> {
    ctx.stack.perform(
        |operands::RawPlay {
             channel,
             velocity,
             note,
         }: operands::RawPlay| {
            Ok(PlayCommand::Raw {
                channel,
                velocity,
                note,
            })
        },
    )
}

/// Timed Play: `!~ channel velocity note length`.
///
/// The length is validated as the Number it is and carried on unread. ADR 0016
/// requires it even where it changes nothing — velocity `00` stops the note
/// whatever length accompanies it — so the operand is extracted here and what
/// it means is decided by the Playback Engine that owns the Ticks it counts.
///
/// One length per element follows from stating the command per element: ADR
/// 0030 gives each element of a widened `!~` its own Note Off at its own
/// length, and the Playback Engine already keys ownership by channel and note,
/// so nothing here or there is added for it.
#[inline(always)]
pub fn timed_play(ctx: &mut Context) -> Result<Performance, Error> {
    ctx.stack.perform(
        |operands::TimedPlay {
             channel,
             velocity,
             note,
             length,
         }: operands::TimedPlay| {
            Ok(PlayCommand::Timed {
                channel,
                velocity,
                note,
                length,
            })
        },
    )
}

#[cfg(test)]
mod test {
    use super::{raw_play, timed_play};
    use crate::{
        Anchor, ArgumentError, Atom, Error, Interpretation, InterpretationError, Interpreter,
        Length, MidiChannel, Note, Parser, Performance, PlayCommand, Sequence, Tick, TickInputs,
        Velocity, interpreter::Context,
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

    /// Pushes `operands` so the Function pops them in signature order.
    fn push_all(ctx: &mut Context, operands: impl IntoIterator<Item = crate::Value>) {
        let operands: Vec<crate::Value> = operands.into_iter().collect();
        for operand in operands.into_iter().rev() {
            ctx.stack.push(operand).unwrap();
        }
    }

    /// A Sequence of Notes, for the operand position a chord is spelled in.
    fn note_sequence(values: impl IntoIterator<Item = u8>) -> Sequence {
        Sequence::new(
            values
                .into_iter()
                .map(|value| Atom::Note(Note::try_from(value).unwrap())),
        )
        .unwrap()
    }

    /// One Raw Play Command, from the bytes a Source would have written.
    fn raw(channel: u8, velocity: u8, note: u8) -> PlayCommand {
        PlayCommand::Raw {
            channel: MidiChannel::try_from(channel).unwrap(),
            velocity: Velocity::try_from(velocity).unwrap(),
            note: Note::try_from(note).unwrap(),
        }
    }

    /// One Timed Play Command, from the bytes a Source would have written.
    fn timed(channel: u8, velocity: u8, note: u8, length: u8) -> PlayCommand {
        PlayCommand::Timed {
            channel: MidiChannel::try_from(channel).unwrap(),
            velocity: Velocity::try_from(velocity).unwrap(),
            note: Note::try_from(note).unwrap(),
            length: Length::from(length),
        }
    }

    #[test]
    fn the_shipped_play_bodies_broadcast_a_sequence_operand() {
        // Pervasion is decided in two places, and the declaration table is only
        // one of them: a body that went back to `Stack::extract` would refuse
        // the Sequence below with `ExpectedAtom` while `RawPlay` still declared
        // `Pervasive`, and every broadcast test written against a test-local
        // restatement of the body would stay green. So these drive the shipped
        // `raw_play` and `timed_play` themselves.
        //
        // One channel and one velocity against three distinct notes: the chord
        // ADR 0030 gives the Source with no new spelling, and distinct notes so
        // a group assembled in reverse is a different answer rather than the
        // same one.
        let mut ctx = context();
        push_all(
            &mut ctx,
            [
                Atom::Number(0x01).into(),
                Atom::Number(0x7F).into(),
                note_sequence([60, 64, 67]).into(),
            ],
        );

        assert_eq!(
            raw_play(&mut ctx).unwrap(),
            Performance::Many(vec![
                raw(0x01, 0x7F, 60),
                raw(0x01, 0x7F, 64),
                raw(0x01, 0x7F, 67),
            ])
        );

        // And the four-operand Function, whose extra scalar repeats across
        // every element exactly as the other two do.
        let mut ctx = context();
        push_all(
            &mut ctx,
            [
                Atom::Number(0x02).into(),
                Atom::Number(0x40).into(),
                note_sequence([60, 64, 67]).into(),
                Atom::Number(0x08).into(),
            ],
        );

        assert_eq!(
            timed_play(&mut ctx).unwrap(),
            Performance::Many(vec![
                timed(0x02, 0x40, 60, 0x08),
                timed(0x02, 0x40, 64, 0x08),
                timed(0x02, 0x40, 67, 0x08),
            ])
        );
    }

    #[test]
    fn a_domain_fault_mid_sequence_leaves_the_shipped_play_bodies_with_no_command() {
        // The all-or-nothing rule through the Functions that actually ship,
        // rather than through a restatement of them. The out-of-domain velocity
        // is the second of three, so a body that handed each command on as it
        // bound the element would already have sounded the first note. Nothing
        // is answered at all, which is the only thing that keeps a partly
        // sounded chord from reaching the Playback Engine.
        let mut ctx = context();
        push_all(
            &mut ctx,
            [
                Atom::Number(0x01).into(),
                Sequence::new([Atom::Number(0x40), Atom::Number(0x80), Atom::Number(0x50)])
                    .unwrap()
                    .into(),
                note_sequence([60, 64, 67]).into(),
            ],
        );

        assert!(matches!(
            raw_play(&mut ctx),
            Err(Error::Interpretation(InterpretationError::MidiDataByte {
                role: "velocity",
                value: 0x80
            }))
        ));

        let mut ctx = context();
        push_all(
            &mut ctx,
            [
                Sequence::new([Atom::Number(0x00), Atom::Number(0x10), Atom::Number(0x02)])
                    .unwrap()
                    .into(),
                Atom::Number(0x7F).into(),
                note_sequence([60, 64, 67]).into(),
                Atom::Number(0x08).into(),
            ],
        );

        assert!(matches!(
            timed_play(&mut ctx),
            Err(Error::Interpretation(InterpretationError::MidiChannel(
                0x10
            )))
        ));
    }

    /// Pins the Play arity contract before issue 04 replaces the placeholder.
    /// See `.scratch/source-playback-engine/issues/04-interpret-terminal-play-functions-into-play-commands.md`
    #[test]
    fn test_raw_play_consumes_exactly_three_arguments() {
        let mut ctx = context();

        // A fourth atom below the three arguments must survive untouched
        ctx.stack.push(Atom::Char('z')).unwrap();
        ctx.stack
            .push(Atom::Note(crate::Note::try_from(60).unwrap()))
            .unwrap(); // n
        ctx.stack.push(Atom::Number(0x7F)).unwrap(); // v
        ctx.stack.push(Atom::Number(0x0)).unwrap(); // c

        let result = raw_play(&mut ctx).unwrap();

        assert_eq!(
            result,
            Performance::One(PlayCommand::Raw {
                channel: MidiChannel::try_from(0).unwrap(),
                velocity: Velocity::try_from(0x7F).unwrap(),
                note: Note::try_from(60).unwrap(),
            })
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
            .push(Atom::Note(crate::Note::try_from(60).unwrap()))
            .unwrap();
        ctx.stack.push(Atom::Number(0x02)).unwrap();
        ctx.stack.push(Atom::Number(0x01)).unwrap();

        let expected = PlayCommand::Raw {
            channel: MidiChannel::try_from(0x01).unwrap(),
            velocity: Velocity::try_from(0x02).unwrap(),
            note: Note::try_from(60).unwrap(),
        };

        assert_eq!(raw_play(&mut ctx).unwrap(), Performance::One(expected));

        // The same claim from Source text, which adds the parse and the
        // right-to-left walk to what the extraction alone proves: `!>` reads
        // channel, then velocity, then note, left to right in the Cells.
        assert_eq!(
            interpret("!>0102C4").unwrap(),
            Interpretation::Play(Performance::One(expected))
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
        ctx.stack.push(Atom::Number(0x04)).unwrap();
        ctx.stack
            .push(Atom::Note(crate::Note::try_from(60).unwrap()))
            .unwrap();
        ctx.stack.push(Atom::Number(0x02)).unwrap();
        ctx.stack.push(Atom::Number(0x01)).unwrap();

        let expected = PlayCommand::Timed {
            channel: MidiChannel::try_from(0x01).unwrap(),
            velocity: Velocity::try_from(0x02).unwrap(),
            note: Note::try_from(60).unwrap(),
            length: Length::from(0x04),
        };

        assert_eq!(timed_play(&mut ctx).unwrap(), Performance::One(expected));

        // And the same claim from Source text, which adds the parse and the
        // right-to-left walk: `!~` reads channel, velocity, note, then length,
        // left to right in the Cells.
        assert_eq!(
            interpret("!~0102C404").unwrap(),
            Interpretation::Play(Performance::One(expected))
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
                ctx.stack.push(*argument).unwrap();
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
        for channel in 0x10..=u8::MAX {
            assert!(
                matches!(
                    interpret(&format!("!~{channel:02X}7FC401")),
                    Err(Error::Interpretation(InterpretationError::MidiChannel(value)))
                        if value == channel
                ),
                "channel {channel:02X}"
            );
        }
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
                Interpretation::Play(Performance::One(PlayCommand::Timed {
                    channel: MidiChannel::try_from(0).unwrap(),
                    velocity: Velocity::try_from(0x7F).unwrap(),
                    note: Note::try_from(60).unwrap(),
                    length: Length::from(length),
                })),
                "length {length:02X}"
            );
        }
    }

    #[test]
    fn test_raw_play_requires_three_arguments() {
        for found in 0..3 {
            let mut ctx = context();
            for _ in 0..found {
                ctx.stack.push(Atom::Number(1)).unwrap();
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
                ctx.stack.push(argument).unwrap();
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
                ctx.stack.push(argument).unwrap();
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
                ctx.stack.push(argument).unwrap();
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
