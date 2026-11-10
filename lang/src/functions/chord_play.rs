use crate::{
    ChordCommands, Error, Function,
    atom::operands::{RawChordPlay, TimedChordPlay},
    chord::ChordOperands,
    interpreter::Context,
};

/// Raw Chord Play: `!: channel velocity root set voicing inversion`.
///
/// One Raw Play Command per tone, lowest first. Channel and velocity take Raw
/// Play's domains, declared beside their roles, so a Chord Play that diagnoses
/// has built no command.
#[inline(always)]
pub fn raw_chord_play(ctx: &mut Context) -> Result<ChordCommands, Error> {
    ctx.stack.perform(
        |RawChordPlay {
             channel,
             velocity,
             root,
             set,
             voicing,
             inversion,
         }: RawChordPlay| {
            ChordCommands::build(ChordOperands {
                function: Function::RawChordPlay,
                channel,
                velocity,
                root,
                set,
                voicing,
                inversion,
                length: None,
            })
        },
    )
}

/// Timed Chord Play: `!# channel velocity root set voicing inversion length`.
///
/// One Timed Play Command per tone, lowest first, each carrying the length
/// unread, as Timed Play's does: velocity `00` and length `00` mean for each
/// tone what they mean for one note, and the Playback Engine decides both.
#[inline(always)]
pub fn timed_chord_play(ctx: &mut Context) -> Result<ChordCommands, Error> {
    ctx.stack.perform(
        |TimedChordPlay {
             channel,
             velocity,
             root,
             set,
             voicing,
             inversion,
             length,
         }: TimedChordPlay| {
            ChordCommands::build(ChordOperands {
                function: Function::TimedChordPlay,
                channel,
                velocity,
                root,
                set,
                voicing,
                inversion,
                length: Some(length),
            })
        },
    )
}

#[cfg(test)]
mod test {
    use super::{raw_chord_play, timed_chord_play};
    use crate::{
        Anchor, ArgumentError, Atom, ChordCommands, Error, Function, Interpretation,
        InterpretationError, Interpreter, IntervalSet, Length, MidiChannel, Note, PlayCommand,
        Tick, TickInputs, Velocity, Voicing, interpreter::Context,
    };

    fn inputs() -> TickInputs {
        TickInputs::new(Tick::ZERO, Anchor::new(0, 0))
    }

    /// Evaluates Source text for one Function over Operand Literals.
    fn interpret(source: &str) -> Result<Interpretation, Error> {
        crate::interpret_source(source)
    }

    /// Exercises Function dispatch with resolved operands in signature order.
    fn evaluate(function: Function, operands: &[Atom]) -> Result<Interpretation, Error> {
        Interpreter::execute_function(function, operands.to_vec(), inputs().into())
    }

    type Body = fn(&mut Context) -> Result<ChordCommands, Error>;

    /// Calls a shipped body with `operands` on the stack in signature order,
    /// so an arity claim reaches the body rather than `execute_function`'s
    /// guard in front of it.
    fn call_body(body: Body, operands: &[Atom]) -> Result<ChordCommands, Error> {
        let mut ctx = Context::new(inputs().into(), 8);
        for operand in operands.iter().rev() {
            ctx.stack.push(*operand).unwrap();
        }
        body(&mut ctx)
    }

    fn note(value: u8) -> Note {
        Note::try_from(value).unwrap()
    }

    /// The Play Commands a chord answer holds, in the order they are sent.
    fn commands(answer: Interpretation) -> Vec<PlayCommand> {
        match answer {
            Interpretation::Chord(chord) => chord.commands().collect(),
            other => panic!("{other:?} is not a chord"),
        }
    }

    /// The MIDI note values a chord answer sounds, in the order they are sent.
    fn tones(source: &str) -> Vec<u8> {
        match interpret(source) {
            Ok(Interpretation::Chord(chord)) => {
                chord.tones().iter().map(|note| note.value()).collect()
            }
            other => panic!("{source} answered {other:?}"),
        }
    }

    fn raw(channel: u8, velocity: u8, note: u8) -> PlayCommand {
        PlayCommand::Raw {
            channel: MidiChannel::try_from(channel).unwrap(),
            velocity: Velocity::try_from(velocity).unwrap(),
            note: self::note(note),
        }
    }

    fn timed(channel: u8, velocity: u8, note: u8, length: u8) -> PlayCommand {
        PlayCommand::Timed {
            channel: MidiChannel::try_from(channel).unwrap(),
            velocity: Velocity::try_from(velocity).unwrap(),
            note: self::note(note),
            length: Length::from(length),
        }
    }

    /// A well-typed Raw Chord Play's operands: C4 major seventh, close, root
    /// position.
    fn raw_operands() -> [Atom; 6] {
        [
            Atom::Number(0x01),
            Atom::Number(0x02),
            Atom::Note(note(60)),
            Atom::IntervalSet(IntervalSet::MajorSeventh),
            Atom::Voicing(Voicing::Close),
            Atom::Number(0x00),
        ]
    }

    /// The same chord as a Timed Chord Play of length `05`.
    fn timed_operands() -> [Atom; 7] {
        let [channel, velocity, root, set, voicing, inversion] = raw_operands();
        [
            channel,
            velocity,
            root,
            set,
            voicing,
            inversion,
            Atom::Number(0x05),
        ]
    }

    #[test]
    fn raw_chord_play_carries_each_operand_into_the_role_its_signature_names() {
        // Differing values in every Number position, so a declaration that
        // transposed two same-token roles still compiles and is caught here:
        // inversion `01` moves C4 above the seventh, which a channel or a
        // velocity of `01` cannot.
        let mut operands = raw_operands();
        operands[5] = Atom::Number(0x01);

        let expected = vec![raw(1, 2, 64), raw(1, 2, 67), raw(1, 2, 71), raw(1, 2, 72)];

        assert_eq!(
            commands(evaluate(Function::RawChordPlay, &operands).unwrap()),
            expected
        );
        // From Source text, which adds the parse and the right-to-left walk:
        // `!:` reads channel, velocity, root, set, voicing, then inversion.
        assert_eq!(commands(interpret("!:0102C4M7CL01").unwrap()), expected);
    }

    #[test]
    fn timed_chord_play_carries_each_operand_into_the_role_its_signature_names() {
        // Channel `01`, velocity `02`, inversion `01` and length `05` all
        // differ, so no transposition among the four Number roles goes
        // unnoticed.
        let mut operands = timed_operands();
        operands[5] = Atom::Number(0x01);

        let expected = vec![
            timed(1, 2, 64, 5),
            timed(1, 2, 67, 5),
            timed(1, 2, 71, 5),
            timed(1, 2, 72, 5),
        ];

        assert_eq!(
            commands(evaluate(Function::TimedChordPlay, &operands).unwrap()),
            expected
        );
        assert_eq!(commands(interpret("!#0102C4M7CL0105").unwrap()), expected);
    }

    #[test]
    fn each_chord_play_requires_exactly_its_declared_operands() {
        // Each well-typed prefix, so what is missing is the count rather than
        // a type, driven against the bodies that own the arity demand.
        let raw = raw_operands();
        for found in 0..raw.len() {
            let error = call_body(raw_chord_play, &raw[..found]).unwrap_err();
            assert!(
                matches!(error, Error::Argument(ArgumentError::Arity { expected: 6, found: f }) if f == found),
                "{found} argument(s) gave {error:?}"
            );
        }
        let timed = timed_operands();
        for found in 0..timed.len() {
            let error = call_body(timed_chord_play, &timed[..found]).unwrap_err();
            assert!(
                matches!(error, Error::Argument(ArgumentError::Arity { expected: 7, found: f }) if f == found),
                "{found} argument(s) gave {error:?}"
            );
        }

        // One operand too many is refused at dispatch for both.
        let mut extra = timed_operands().to_vec();
        extra.push(Atom::Number(0));
        assert!(matches!(
            evaluate(Function::TimedChordPlay, &extra),
            Err(Error::Argument(ArgumentError::Arity {
                expected: 7,
                found: 8
            }))
        ));
        assert!(matches!(
            evaluate(Function::RawChordPlay, &extra[..7]),
            Err(Error::Argument(ArgumentError::Arity {
                expected: 6,
                found: 7
            }))
        ));
    }

    #[test]
    fn channel_velocity_and_length_take_plays_domains() {
        for channel in 0x10..=u8::MAX {
            for source in [
                format!("!:{channel:02X}7FC4M7CL00"),
                format!("!#{channel:02X}7FC4M7CL0001"),
            ] {
                assert!(
                    matches!(
                        interpret(&source),
                        Err(Error::Interpretation(InterpretationError::MidiChannel(value)))
                            if value == channel
                    ),
                    "{source}"
                );
            }
        }
        for velocity in 0x80..=u8::MAX {
            for source in [
                format!("!:00{velocity:02X}C4M7CL00"),
                format!("!#00{velocity:02X}C4M7CL0001"),
            ] {
                assert!(
                    matches!(
                        interpret(&source),
                        Err(Error::Interpretation(InterpretationError::MidiDataByte {
                            role: "velocity",
                            value,
                        })) if value == velocity
                    ),
                    "{source}"
                );
            }
        }
        // Every byte is a length, `00` included: what it means is the
        // Playback Engine's, as it is for Timed Play.
        for length in 0..=u8::MAX {
            assert_eq!(
                commands(interpret(&format!("!#007FC4M_CL00{length:02X}")).unwrap()),
                vec![
                    timed(0, 0x7F, 60, length),
                    timed(0, 0x7F, 64, length),
                    timed(0, 0x7F, 67, length),
                ],
                "length {length:02X}"
            );
        }
        // Velocity `00` is carried to every tone as Play's explicit stop.
        assert_eq!(
            commands(interpret("!:0000C4M_CL00").unwrap()),
            vec![raw(0, 0, 60), raw(0, 0, 64), raw(0, 0, 67)]
        );
    }

    #[test]
    fn every_operand_refuses_an_atom_of_another_type() {
        // A Note where a Number is declared, a Number where a Note is, and a
        // Number or a Note where an Interval Set or a Voicing is: Play infers
        // no type from another, and a type fault precedes every domain fault.
        let wrong = [
            (0, Atom::Note(note(1))),
            (1, Atom::Note(note(1))),
            (2, Atom::Number(60)),
            (3, Atom::Number(0x69)),
            (3, Atom::Note(note(60))),
            (3, Atom::Voicing(Voicing::Close)),
            (4, Atom::Note(note(38))),
            (4, Atom::Number(0)),
            (4, Atom::IntervalSet(IntervalSet::MajorTriad)),
            (5, Atom::Note(note(1))),
            (6, Atom::Note(note(1))),
        ];
        for (position, atom) in wrong {
            let mut operands = timed_operands();
            operands[position] = atom;
            assert!(
                matches!(
                    evaluate(Function::TimedChordPlay, &operands),
                    Err(Error::Type(_))
                ),
                "{atom:?} in operand {position} was accepted"
            );
            if position < 6 {
                let mut operands = raw_operands();
                operands[position] = atom;
                assert!(
                    matches!(
                        evaluate(Function::RawChordPlay, &operands),
                        Err(Error::Type(_))
                    ),
                    "{atom:?} in Raw operand {position} was accepted"
                );
            }
        }
        // A name that also spells a Number or a Note is read by the slot it
        // stands in: `69` is a six-nine chord and `D2` a drop voicing here.
        assert_eq!(tones("!:007FC469CL00"), [60, 64, 67, 69, 74]);
        assert_eq!(tones("!:007FC4M7D200"), [55, 60, 64, 71]);
        // A half-blank or unknown name is refused at the parse.
        for source in ["!:007FC4M CL00", "!:007FC4M7C 00", "!:007FC4MXCL00"] {
            assert!(
                matches!(interpret(source), Err(Error::Type(_))),
                "{source}"
            );
        }
    }

    #[test]
    fn every_voicing_alone_and_with_one_inversion_gives_the_tones_it_names() {
        // C4 major seventh, C E G B, worked by hand. The Voicing removes, the
        // inversion moves the lowest tone up an octave, and then the Voicing
        // moves tones counted on the inverted chord. Tones are sent lowest
        // first whatever their Interval Set position.
        let cases: [(&str, [&[u8]; 2]); 8] = [
            // C E G B; E G B C.
            ("CL", [&[60, 64, 67, 71], &[64, 67, 71, 72]]),
            // 2nd and 4th raised: C G E B; of E G B C, G and C raised.
            ("OP", [&[60, 67, 76, 83], &[64, 71, 79, 84]]),
            // 2nd from the top dropped: G; of E G B C, B.
            ("D2", [&[55, 60, 64, 71], &[59, 64, 67, 72]]),
            // 3rd from the top dropped: E; of E G B C, G.
            ("D3", [&[52, 60, 67, 71], &[55, 64, 71, 72]]),
            // 2nd and 3rd from the top: G and E; of E G B C, B and G.
            ("D5", [&[52, 55, 60, 71], &[55, 59, 64, 72]]),
            // 2nd and 4th from the top: G and C; of E G B C, B and E.
            ("D6", [&[48, 55, 64, 71], &[52, 59, 67, 72]]),
            // 1st, 2nd and 4th kept: C E B, then inverted: E B C.
            ("SH", [&[60, 64, 71], &[64, 71, 72]]),
            // 1st removed: E G B, then inverted: G B E.
            ("RL", [&[64, 67, 71], &[67, 71, 76]]),
        ];
        for (voicing, [root_position, inverted]) in cases {
            assert_eq!(
                tones(&format!("!:007FC4M7{voicing}00")),
                root_position,
                "{voicing}"
            );
            assert_eq!(
                tones(&format!("!:007FC4M7{voicing}01")),
                inverted,
                "{voicing} inverted"
            );
        }
    }

    #[test]
    fn removal_precedes_inversion_and_inversion_precedes_movement() {
        // Each pair below differs from what the other order gives, so the
        // order is what the expected tones pin.
        //
        // Inverting before a Shell would keep E G C (64 67 72).
        assert_eq!(tones("!:007FC4M7SH01"), [64, 71, 72]);
        // Inverting before Rootless would leave G B C (67 71 72).
        assert_eq!(tones("!:007FC4M7RL01"), [67, 71, 76]);
        // Dropping before inverting gives C E G B again (60 64 67 71).
        assert_eq!(tones("!:007FC4M7D201"), [59, 64, 67, 72]);
        // Opening before inverting gives E G B C an octave up (64 67 76 83).
        assert_eq!(tones("!:007FC4M7OP01"), [64, 71, 79, 84]);
    }

    #[test]
    fn each_inversion_moves_the_lowest_tone_up_an_octave() {
        assert_eq!(tones("!:007FC4M_CL00"), [60, 64, 67]);
        assert_eq!(tones("!:007FC4M_CL01"), [64, 67, 72]);
        assert_eq!(tones("!:007FC4M_CL02"), [67, 72, 76]);
        assert_eq!(tones("!:007FC4M_CL03"), [72, 76, 79]);
        // A chord wider than an octave: the lowest tone moved up an octave
        // can still sit below the top, and the next inversion moves the new
        // lowest, not the old second.
        assert_eq!(tones("!:007FC4MDCL00"), [60, 64, 67, 71, 74, 81]);
        assert_eq!(tones("!:007FC4MDCL01"), [64, 67, 71, 72, 74, 81]);
        assert_eq!(tones("!:007FC4MDCL02"), [67, 71, 72, 74, 76, 81]);
        // Inversion is any Number: one that carries a tone past `7F`
        // diagnoses the whole chord as any out-of-range tone does.
        assert!(matches!(
            interpret("!:007FC/M_CLFF"),
            Err(Error::Interpretation(InterpretationError::ChordToneOutOfRange {
                function: Function::RawChordPlay,
                ..
            }))
        ));
    }

    #[test]
    fn a_twelve_tone_set_sounds_all_twelve_lowest_first() {
        assert_eq!(
            commands(interpret("!#037FC4chCL0001").unwrap()),
            (60..72).map(|note| timed(3, 0x7F, note, 1)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn too_few_tones_for_a_voicing_diagnoses_the_whole_chord() {
        // Every Interval Set against every Voicing, against the counts the
        // Voicings' definitions need, stated here rather than read from them.
        let needed = [
            (Voicing::Close, 1),
            (Voicing::Open, 2),
            (Voicing::Drop2, 2),
            (Voicing::Drop3, 3),
            (Voicing::Drop5, 3),
            (Voicing::Drop6, 4),
            (Voicing::Shell, 4),
            (Voicing::Rootless, 2),
        ];
        for &set in IntervalSet::ALL {
            for (voicing, needs) in needed {
                let mut operands = raw_operands();
                operands[2] = Atom::Note(note(48));
                operands[3] = Atom::IntervalSet(set);
                operands[4] = Atom::Voicing(voicing);
                let answer = evaluate(Function::RawChordPlay, &operands);
                if set.semitones().len() < needs {
                    assert!(
                        matches!(
                            answer,
                            Err(Error::Interpretation(InterpretationError::TooFewTones {
                                set: s, voicing: v, needed: n,
                            })) if s == set && v == voicing && n == needs
                        ),
                        "{set:?} {voicing:?} gave {answer:?}"
                    );
                } else {
                    assert!(
                        matches!(answer, Ok(Interpretation::Chord(_))),
                        "{set:?} {voicing:?} gave {answer:?}"
                    );
                }
            }
        }
        assert_eq!(
            interpret("!:007FC4M_SH00").unwrap_err().to_string(),
            "Voicing SH needs 4 tones and Interval Set M_ has 3, so no tone is sent"
        );
    }

    #[test]
    fn a_tone_outside_the_note_range_diagnoses_the_whole_chord() {
        // G9 is `7F`, so its major third is past the range; C/ is `00`, so a
        // dropped tone falls below it. Neither answer holds any command.
        for (source, function, value) in [
            ("!:007FG9M_CL00", Function::RawChordPlay, 0x83),
            ("!#007FG9M_CL0001", Function::TimedChordPlay, 0x83),
            ("!:007FC/M7D200", Function::RawChordPlay, -5),
            ("!:007FC/M7D600", Function::RawChordPlay, -12),
        ] {
            assert!(
                matches!(
                    interpret(source),
                    Err(Error::Interpretation(InterpretationError::ChordToneOutOfRange {
                        function: f, value: v,
                    })) if f == function && v == value
                ),
                "{source}"
            );
        }
        // At the boundary: the top tone at exactly `7F` sounds.
        assert_eq!(tones("!:007FC9M_CL00"), [120, 124, 127]);
        assert_eq!(
            interpret("!:007FC/M7D200").unwrap_err().to_string(),
            "!: would sound -05, outside the Note range 00–7F, so no tone is sent"
        );
    }
}
