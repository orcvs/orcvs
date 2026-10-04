pub(crate) mod jump;
pub(crate) mod math;
pub(crate) mod numeric_conversion;
pub(crate) mod tick;
use crate::{Error, PlayCommand, atom::operands, interpreter::Context};

// Each body states one Play Command for the operands `Stack::perform` binds.
// A chord is several of these roots activated by one Bang, each answering its
// own command, so no body here knows about more than one note.

/// Raw Play: `!> channel velocity note`.
///
/// There is no validation call here. Each operand's domain is declared beside
/// its role in `define_functions!` and converted as each operand binds, so a
/// Function that diagnoses has produced no Play Command at all, and a new MIDI
/// terminal Function inherits its validation from its declaration rather than
/// from a body that remembers to ask for it.
#[inline(always)]
pub fn raw_play(ctx: &mut Context) -> Result<PlayCommand, Error> {
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
#[inline(always)]
pub fn timed_play(ctx: &mut Context) -> Result<PlayCommand, Error> {
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

/// Monophonic Play: `!% channel velocity note length`.
///
/// The operand shape is Timed Play's, so the body is too. What differs is
/// entirely downstream: this command owns a channel rather than a note, and
/// the Playback Engine that counts Ticks is what knows the difference. A body
/// that tried to say so here would be interpreting musical intent inside a
/// Tick Plan, which is the seam ADR 0001 draws.
#[inline(always)]
pub fn monophonic_play(ctx: &mut Context) -> Result<PlayCommand, Error> {
    ctx.stack.perform(
        |operands::MonophonicPlay {
             channel,
             velocity,
             note,
             length,
         }: operands::MonophonicPlay| {
            Ok(PlayCommand::Mono {
                channel,
                velocity,
                note,
                length,
            })
        },
    )
}

/// Control Change: `!c channel controller value`.
///
/// No validation call here either, and no wire byte: what `0xB0` does with a
/// controller and its value is the output adapter's, and what makes each
/// operand legal is declared beside its role in `define_functions!`. The two
/// data bytes share a domain and differ in type, so this body cannot hand one
/// role's operand to the other even by writing the fields out of order.
#[inline(always)]
pub fn control_change(ctx: &mut Context) -> Result<PlayCommand, Error> {
    ctx.stack.perform(
        |operands::ControlChange {
             channel,
             controller,
             value,
         }: operands::ControlChange| {
            Ok(PlayCommand::ControlChange {
                channel,
                controller,
                value,
            })
        },
    )
}

/// Pitch Bend: `!b channel lsb msb`.
///
/// The halves are carried as the Source wrote them, in the order the wire
/// takes them. Combining them into a fourteen-bit bend here would be a
/// scaling decision ADR 0016 refuses and would leave the adapter splitting
/// apart what this had just joined.
#[inline(always)]
pub fn pitch_bend(ctx: &mut Context) -> Result<PlayCommand, Error> {
    ctx.stack.perform(
        |operands::PitchBend { channel, lsb, msb }: operands::PitchBend| {
            Ok(PlayCommand::PitchBend { channel, lsb, msb })
        },
    )
}

#[cfg(test)]
mod test {
    use super::{control_change, monophonic_play, pitch_bend, raw_play, timed_play};
    use crate::{
        Anchor, ArgumentError, Atom, BendLsb, BendMsb, ControlValue, Controller, Error, Function,
        Interpretation, InterpretationError, Interpreter, Length, MidiChannel, Note, PlayCommand,
        Tick, TickInputs, Velocity, interpreter::Context,
    };

    ///
    /// The Tick inputs for a test about operands rather than about time or
    /// Position: the first Tick of a Playback run, at the Grid origin.
    ///
    fn inputs() -> TickInputs {
        TickInputs::new(Tick::ZERO, Anchor::new(0, 0))
    }

    /// Evaluates Source text for one Function over Operand Literals.
    fn interpret(source: &str) -> Result<Interpretation, Error> {
        crate::interpret_source(source)
    }

    /// A lifetime Play's operands with the Number `3C` where the Note stands:
    /// what `.vC4` answers there.
    fn midi_play_with_a_number_note() -> [Atom; 4] {
        [0x00, 0x7F, 0x3C, 0x01].map(Atom::Number)
    }

    /// Exercises Function dispatch with resolved operands in signature order.
    fn evaluate(function: Function, operands: &[Atom]) -> Result<Interpretation, Error> {
        Interpreter::execute_function(function, operands.to_vec(), inputs().into())
    }

    /// A terminal body, so the two spellings that share a claim can be stated
    /// once and asserted over rather than written out for each.
    type Terminal = fn(&mut Context) -> Result<PlayCommand, Error>;

    /// Calls a shipped body with `operands` on the stack in signature order.
    ///
    /// `Interpreter::execute_function` answers a wrong operand count itself,
    /// before it dispatches, so an arity claim made through `evaluate` observes
    /// that guard rather than the Function it names. The bodies own their
    /// arity demand, so the tests that pin it reach them here.
    fn call_body(body: Terminal, operands: &[Atom]) -> Result<PlayCommand, Error> {
        let mut ctx = Context::new(inputs().into(), 4);
        for operand in operands.iter().rev() {
            ctx.stack.push(*operand).unwrap();
        }
        body(&mut ctx)
    }

    /// This internal test observes Stack consumption, which the Evaluator
    /// interface intentionally hides from its callers.
    #[test]
    fn test_raw_play_consumes_exactly_three_arguments() {
        let mut ctx = Context::new(inputs().into(), 4);

        // A fourth atom below the three arguments must survive untouched
        ctx.stack.push(Atom::Bang).unwrap();
        ctx.stack
            .push(Atom::Note(crate::Note::try_from(60).unwrap()))
            .unwrap(); // n
        ctx.stack.push(Atom::Number(0x7F)).unwrap(); // v
        ctx.stack.push(Atom::Number(0x0)).unwrap(); // c

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
        assert_eq!(ctx.stack.pop_value(), Some(Atom::Bang));
        assert_eq!(ctx.stack.pop_value(), None);
    }

    #[test]
    fn play_carries_each_operand_into_the_role_its_signature_names() {
        // 01 and 02 are legal as a channel and as a data byte, and a complete
        // role swap inside the declaration carries each name with its own type,
        // so it still compiles and every operand still lands in a legal domain.
        // Only differing operand values separate a correct declaration from a
        // transposed one, which is why this test cannot be replaced by the
        // domain types it sits beside.
        let operands = [
            Atom::Number(0x01),
            Atom::Number(0x02),
            Atom::Note(Note::try_from(60).unwrap()),
        ];

        let expected = PlayCommand::Raw {
            channel: MidiChannel::try_from(0x01).unwrap(),
            velocity: Velocity::try_from(0x02).unwrap(),
            note: Note::try_from(60).unwrap(),
        };

        assert_eq!(
            evaluate(Function::RawPlay, &operands).unwrap(),
            Interpretation::Play(expected)
        );

        // The same claim from Source text, which adds the parse and the
        // right-to-left walk to what resolved-operand evaluation proves:
        // `!>` reads channel, then velocity, then note, left to right.
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
        let operands = [
            Atom::Number(0x01),
            Atom::Number(0x02),
            Atom::Note(Note::try_from(60).unwrap()),
            Atom::Number(0x04),
        ];

        let expected = PlayCommand::Timed {
            channel: MidiChannel::try_from(0x01).unwrap(),
            velocity: Velocity::try_from(0x02).unwrap(),
            note: Note::try_from(60).unwrap(),
            length: Length::from(0x04),
        };

        assert_eq!(
            evaluate(Function::TimedPlay, &operands).unwrap(),
            Interpretation::Play(expected)
        );

        // And the same claim from Source text, which adds the parse and the
        // right-to-left walk: `!~` reads channel, velocity, note, then length,
        // left to right in the Cells.
        assert_eq!(
            interpret("!~0102C404").unwrap(),
            Interpretation::Play(expected)
        );
    }

    #[test]
    fn monophonic_play_carries_each_operand_into_the_role_its_signature_names() {
        // Four differing values for the reason Timed Play's role test gives:
        // `!%` shares its operand shape, so a transposition inside the
        // declaration compiles and only values that differ separate it from
        // the declaration meant.
        let operands = [
            Atom::Number(0x01),
            Atom::Number(0x02),
            Atom::Note(Note::try_from(60).unwrap()),
            Atom::Number(0x04),
        ];

        let expected = PlayCommand::Mono {
            channel: MidiChannel::try_from(0x01).unwrap(),
            velocity: Velocity::try_from(0x02).unwrap(),
            note: Note::try_from(60).unwrap(),
            length: Length::from(0x04),
        };

        assert_eq!(
            evaluate(Function::MonophonicPlay, &operands).unwrap(),
            Interpretation::Play(expected)
        );

        // And the same claim from Source text: `!%` reads channel, velocity,
        // note, then length, left to right in the Cells.
        assert_eq!(
            interpret("!%0102C404").unwrap(),
            Interpretation::Play(expected)
        );
    }

    #[test]
    fn monophonic_play_requires_four_arguments() {
        // A well-typed prefix at every length, as Timed Play's arity test
        // does: what is missing is the count rather than a type. Driven
        // against the body, which is where the arity demand lives.
        let operands = [
            Atom::Number(0x01),
            Atom::Number(0x02),
            Atom::Note(crate::Note::try_from(60).unwrap()),
            Atom::Number(0x04),
        ];

        for found in 0..4 {
            let error = call_body(monophonic_play, &operands[..found]).unwrap_err();

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
    fn monophonic_play_takes_the_same_operand_domains_as_timed_play() {
        // `!%` has Timed Play's operand shape, so it inherits the
        // domains with it. Each is proven by the operand that leaves it,
        // except the length, which has nothing outside it.
        for channel in 0x10..=u8::MAX {
            assert!(
                matches!(
                    interpret(&format!("!%{channel:02X}7FC401")),
                    Err(Error::Interpretation(InterpretationError::MidiChannel(value)))
                        if value == channel
                ),
                "channel {channel:02X}"
            );
        }
        assert!(matches!(
            interpret("!%0080C401"),
            Err(Error::Interpretation(InterpretationError::MidiDataByte {
                role: "velocity",
                value: 0x80,
            }))
        ));
        assert!(matches!(
            evaluate(Function::MonophonicPlay, &midi_play_with_a_number_note()),
            Err(Error::Type(crate::TypeError::Note(found))) if found == "3C"
        ));

        for length in 0..=u8::MAX {
            assert_eq!(
                interpret(&format!("!%007FC4{length:02X}")).unwrap(),
                Interpretation::Play(PlayCommand::Mono {
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
    fn timed_play_requires_four_arguments() {
        // Each prefix of a well-typed operand list, so what is missing is the
        // count rather than a type: an arity diagnostic must precede every
        // other one, and only a correctly typed prefix can prove it does.
        // That ordering is the body's, so the body is what this drives.
        let operands = [
            Atom::Number(0x01),
            Atom::Number(0x02),
            Atom::Note(crate::Note::try_from(60).unwrap()),
            Atom::Number(0x04),
        ];

        for found in 0..4 {
            let error = call_body(timed_play, &operands[..found]).unwrap_err();

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
        // The domains of `!~`, each proven by the operand that
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
            evaluate(Function::TimedPlay, &midi_play_with_a_number_note()),
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
    fn control_change_carries_each_operand_into_the_role_its_signature_names() {
        // Three differing operand values, for the reason Raw Play's role test
        // gives and for one more that belongs to `!c` alone. Controller and
        // value share a domain, so a declaration that transposed them carries
        // each role name with its own type, compiles, and validates: the role
        // types make the swap unrepresentable in every body that reads the
        // command, and cannot see a transposition of the declaration itself.
        // `01`, `02`, and `03` are legal in all three positions, so nothing
        // but the values separates the declaration meant from its transposition.
        let operands = [Atom::Number(0x01), Atom::Number(0x02), Atom::Number(0x03)];

        let expected = PlayCommand::ControlChange {
            channel: MidiChannel::try_from(0x01).unwrap(),
            controller: Controller::try_from(0x02).unwrap(),
            value: ControlValue::try_from(0x03).unwrap(),
        };

        assert_eq!(
            evaluate(Function::ControlChange, &operands).unwrap(),
            Interpretation::Play(expected)
        );

        // The same claim from Source text, which adds the parse and the
        // right-to-left walk to what resolved-operand evaluation proves:
        // `!c` reads channel, then controller, then value, left to right.
        assert_eq!(
            interpret("!c010203").unwrap(),
            Interpretation::Play(expected)
        );
    }

    #[test]
    fn pitch_bend_carries_each_operand_into_the_role_its_signature_names() {
        // LSB and MSB are the exposed pair here, for the reason controller and
        // value are in the test above: one shared data-byte domain, two roles,
        // and a wire order that a transposition would silently reverse into a
        // bend of an entirely different pitch.
        let operands = [Atom::Number(0x01), Atom::Number(0x02), Atom::Number(0x03)];

        let expected = PlayCommand::PitchBend {
            channel: MidiChannel::try_from(0x01).unwrap(),
            lsb: BendLsb::try_from(0x02).unwrap(),
            msb: BendMsb::try_from(0x03).unwrap(),
        };

        assert_eq!(
            evaluate(Function::PitchBend, &operands).unwrap(),
            Interpretation::Play(expected)
        );

        // And from Source text: `!b` reads channel, then LSB, then MSB, left
        // to right in the Cells, which is also the order they go out on.
        assert_eq!(
            interpret("!b010203").unwrap(),
            Interpretation::Play(expected)
        );
    }

    /// The Source text that puts one byte in one data-byte position.
    type SourceText = fn(u8) -> String;

    /// The command that position must answer with, for that byte.
    type ExpectedCommand = fn(MidiChannel, u8) -> PlayCommand;

    #[test]
    fn control_change_and_pitch_bend_require_exactly_three_arguments() {
        // Each prefix of a well-typed operand list, so what is missing is the
        // count rather than a type: an arity diagnostic must precede every
        // other one, and only a correctly typed prefix can prove it does.
        // That ordering is the body's, so the bodies are what this drives.
        let operands = [Atom::Number(0x01), Atom::Number(0x02), Atom::Number(0x03)];
        for body in [control_change as Terminal, pitch_bend as Terminal] {
            for found in 0..3 {
                let error = call_body(body, &operands[..found]).unwrap_err();

                assert!(
                    matches!(
                        error,
                        Error::Argument(ArgumentError::Arity { expected: 3, found: f }) if f == found
                    ),
                    "{found} argument(s) gave {error:?}"
                );
            }
        }
    }

    #[test]
    fn control_change_and_pitch_bend_reject_a_note_in_every_operand_position() {
        // Every operand of both Functions is declared over a Number, so a Note
        // is what separates the token the parser reads from the domain declared
        // over it. `!>` and `!~` each carry this claim for their own
        // signatures; without it these two are covered for arity and for range
        // but never for the type refusal that has to precede both.
        for function in [Function::ControlChange, Function::PitchBend] {
            for mistyped in 0..3 {
                let mut arguments = [Atom::Number(0x01), Atom::Number(0x02), Atom::Number(0x03)];
                arguments[mistyped] = Atom::Note(crate::Note::try_from(0x03).unwrap());

                assert!(
                    matches!(evaluate(function, &arguments), Err(Error::Type(_))),
                    "a Note in operand {mistyped} was accepted",
                );
            }
        }
    }

    #[test]
    fn control_change_and_pitch_bend_take_a_midi_channel_and_two_data_bytes() {
        // The domains of both spellings, each proven by an operand
        // that leaves them. The data-byte diagnostics are what the role types
        // buy at the Source: two operands of one domain, and a diagnostic that
        // still names which of them the Source wrote out of range.
        for channel in 0x10..=u8::MAX {
            for expression in [
                format!("!c{channel:02X}0203"),
                format!("!b{channel:02X}0203"),
            ] {
                assert!(
                    matches!(
                        interpret(&expression),
                        Err(Error::Interpretation(InterpretationError::MidiChannel(value)))
                            if value == channel
                    ),
                    "{expression}"
                );
            }
        }

        for (expression, expected) in [
            ("!c018003", "controller"),
            ("!c010280", "value"),
            ("!b018003", "lsb"),
            ("!b010280", "msb"),
        ] {
            assert!(
                matches!(
                    interpret(expression),
                    Err(Error::Interpretation(InterpretationError::MidiDataByte {
                        role,
                        value: 0x80,
                    })) if role == expected
                ),
                "{expression}"
            );
        }
    }

    #[test]
    fn every_data_byte_reaches_a_control_change_or_pitch_bend_command_unaltered() {
        // ADR 0016 sends direct MIDI values, so the whole accepted domain is
        // enumerated rather than sampled: a scale, a wrap, or a clamp applied
        // to a data byte answers a perfectly legal command and differs only in
        // the value it carries.
        let channel = MidiChannel::try_from(0x01).unwrap();

        // One row per data-byte position: the Source text that puts `byte`
        // there, and the command that position must answer with. A table
        // because it is the same claim four times, and a table is what makes a
        // position that went missing visible.
        let positions: [(SourceText, ExpectedCommand); 4] = [
            (
                |byte| format!("!c01{byte:02X}03"),
                |channel, byte| PlayCommand::ControlChange {
                    channel,
                    controller: Controller::try_from(byte).unwrap(),
                    value: ControlValue::try_from(0x03).unwrap(),
                },
            ),
            (
                |byte| format!("!c0102{byte:02X}"),
                |channel, byte| PlayCommand::ControlChange {
                    channel,
                    controller: Controller::try_from(0x02).unwrap(),
                    value: ControlValue::try_from(byte).unwrap(),
                },
            ),
            (
                |byte| format!("!b01{byte:02X}03"),
                |channel, byte| PlayCommand::PitchBend {
                    channel,
                    lsb: BendLsb::try_from(byte).unwrap(),
                    msb: BendMsb::try_from(0x03).unwrap(),
                },
            ),
            (
                |byte| format!("!b0102{byte:02X}"),
                |channel, byte| PlayCommand::PitchBend {
                    channel,
                    lsb: BendLsb::try_from(0x02).unwrap(),
                    msb: BendMsb::try_from(byte).unwrap(),
                },
            ),
        ];

        for (source_text, expected) in positions {
            for byte in 0..=0x7F {
                let expression = source_text(byte);

                assert_eq!(
                    interpret(&expression).unwrap(),
                    Interpretation::Play(expected(channel, byte)),
                    "{expression}"
                );
            }
        }
    }

    #[test]
    fn test_raw_play_requires_three_arguments() {
        let operands = [Atom::Number(1); 3];
        for found in 0..3 {
            let error = call_body(raw_play, &operands[..found]).unwrap_err();

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
            assert!(matches!(
                evaluate(Function::RawPlay, &arguments),
                Err(Error::Type(_))
            ));
        }
    }

    #[test]
    fn play_rejects_channels_outside_the_midi_range() {
        for channel in 0x10..=u8::MAX {
            let operands = [
                Atom::Number(channel),
                Atom::Number(0x7F),
                Atom::Note(crate::Note::try_from(60).unwrap()),
            ];

            assert!(matches!(
                evaluate(Function::RawPlay, &operands),
                Err(Error::Interpretation(InterpretationError::MidiChannel(value)))
                    if value == channel
            ));
        }
    }

    #[test]
    fn play_rejects_velocities_outside_the_midi_data_byte_range() {
        for velocity in 0x80..=u8::MAX {
            let operands = [
                Atom::Number(0),
                Atom::Number(velocity),
                Atom::Note(crate::Note::try_from(60).unwrap()),
            ];

            assert!(matches!(
                evaluate(Function::RawPlay, &operands),
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
        // for equal operands, and a Turn hands it to `.^`, which expects a
        // Number or a Note.
        assert!(matches!(
            evaluate(Function::ConvertToNote, &[Atom::Bang]),
            Err(Error::Type(crate::TypeError::Numeric(found))) if found == "**"
        ));
    }

    #[test]
    fn an_out_of_domain_operand_produces_no_play_command_at_all() {
        // The domain conversion happens during extraction, so a Play that
        // diagnoses has never constructed a PlayCommand to be discarded. The
        // same holds for every terminal spelling, because the conversion is
        // declared in the table rather than performed by the body.
        assert!(matches!(
            interpret("!>107FC4"),
            Err(Error::Interpretation(InterpretationError::MidiChannel(
                0x10
            )))
        ));
        assert!(matches!(
            interpret("!c010280"),
            Err(Error::Interpretation(InterpretationError::MidiDataByte {
                role: "value",
                value: 0x80,
            }))
        ));
        assert!(matches!(
            interpret("!b108001"),
            Err(Error::Interpretation(InterpretationError::MidiChannel(
                0x10
            )))
        ));
    }
}
