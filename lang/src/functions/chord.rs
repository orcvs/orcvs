//! Chord Play: one Expression that sounds every tone of a Chord.

use arrayvec::ArrayVec;

use crate::{
    Error, Function, InterpretationError, IntervalSet, Length, MidiChannel, Note, PlayCommand,
    Velocity, Voicing, atom::operands, functions::pitch::note_at, interpreter::Context,
};

/// The most tones any Interval Set names, and so the most Play Commands one
/// Chord Play answers. Read off the Interval Set table so a wider name widens
/// every buffer that holds tones by being declared.
pub(crate) const MAX_TONES: usize = {
    let mut widest = 0;
    let mut index = 0;
    while index < IntervalSet::ALL.len() {
        let tones = IntervalSet::ALL[index].semitones().len();
        if tones > widest {
            widest = tones;
        }
        index += 1;
    }
    widest
};

/// The semitones one tone moves when it moves an octave.
const OCTAVE: i16 = 12;

/// What one Chord Play's Turn performs: the operands its tones share, held
/// once, and its tones, lowest first.
///
/// It exists only once the whole chord has been built, so holding one means
/// every tone is a Note. The tones are inline, and the shared operands are not
/// repeated per tone, so every Turn's answer stays small and a chord asks the
/// allocator for nothing.
#[derive(Clone, Debug, PartialEq)]
pub struct ChordTones {
    channel: MidiChannel,
    velocity: Velocity,
    /// The lifetime every tone carries, or `None` for a Raw Chord Play.
    length: Option<Length>,
    notes: ArrayVec<Note, MAX_TONES>,
}

impl ChordTones {
    /// One ordinary Play Command per tone, lowest first: Raw for a Raw Chord
    /// Play and Timed for a Timed one.
    ///
    /// The kind is chosen once for the whole chord, so the per-tone step only
    /// builds a command.
    pub fn commands(&self) -> impl ExactSizeIterator<Item = PlayCommand> + use<'_> {
        let (channel, velocity) = (self.channel, self.velocity);
        let notes = self.notes.iter();
        match self.length {
            None => Commands::Raw(notes.map(move |&note| PlayCommand::Raw {
                channel,
                velocity,
                note,
            })),
            Some(length) => Commands::Timed(notes.map(move |&note| PlayCommand::Timed {
                channel,
                velocity,
                note,
                length,
            })),
        }
    }
}

/// The tones of one chord as Raw or as Timed Play Commands: two iterators of
/// one item type behind a single one, so [`ChordTones::commands`] answers
/// without boxing.
enum Commands<R, T> {
    Raw(R),
    Timed(T),
}

impl<R, T> Iterator for Commands<R, T>
where
    R: Iterator<Item = PlayCommand>,
    T: Iterator<Item = PlayCommand>,
{
    type Item = PlayCommand;

    fn next(&mut self) -> Option<PlayCommand> {
        match self {
            Self::Raw(commands) => commands.next(),
            Self::Timed(commands) => commands.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Raw(commands) => commands.size_hint(),
            Self::Timed(commands) => commands.size_hint(),
        }
    }
}

impl<R, T> ExactSizeIterator for Commands<R, T>
where
    R: ExactSizeIterator<Item = PlayCommand>,
    T: ExactSizeIterator<Item = PlayCommand>,
{
}

/// Raw Chord Play: `!: channel velocity root set voicing inversion`.
///
/// One Raw Play Command per tone, so velocity `00` stops a tone exactly as it
/// stops a note, and the Playback Engine has nothing of its own to learn.
#[inline(always)]
pub fn raw_chord_play(ctx: &mut Context) -> Result<ChordTones, Error> {
    let operands::RawChordPlay {
        channel,
        velocity,
        root,
        set,
        voicing,
        inversion,
    } = ctx.stack.extract()?;
    Ok(ChordTones {
        channel,
        velocity,
        length: None,
        notes: tones(Function::RawChordPlay, root, set, voicing, inversion)?,
    })
}

/// Timed Chord Play: `!# channel velocity root set voicing inversion length`.
///
/// One Timed Play Command per tone, so each tone owns its channel and note and
/// stops at its own Note Off, as a Timed Play does.
#[inline(always)]
pub fn timed_chord_play(ctx: &mut Context) -> Result<ChordTones, Error> {
    let operands::TimedChordPlay {
        channel,
        velocity,
        root,
        set,
        voicing,
        inversion,
        length,
    } = ctx.stack.extract()?;
    Ok(ChordTones {
        channel,
        velocity,
        length: Some(length),
        notes: tones(Function::TimedChordPlay, root, set, voicing, inversion)?,
    })
}

/// The tones a Chord Play sends, lowest first, or why the whole chord is
/// refused.
///
/// The order is fixed: the Voicing removes tones, the inversion then moves the
/// lowest tone up an octave `inversion` times, and the Voicing then moves the
/// tones it moves. A tone is checked against the Note domain only once every
/// move has been made, because a drop or an inversion can pass through a value
/// outside it on the way to one inside. `function` names the Chord Play in
/// the diagnostic for a tone that ends outside it.
fn tones(
    function: Function,
    root: Note,
    set: IntervalSet,
    voicing: Voicing,
    inversion: u8,
) -> Result<ArrayVec<Note, MAX_TONES>, Error> {
    let semitones = set.semitones();
    if semitones.len() < voicing.tones_needed() {
        return Err(InterpretationError::TooFewTones {
            function,
            set,
            voicing,
        }
        .into());
    }

    // Every semitone table is listed lowest first, so filtering by position
    // keeps the tones sorted. `i16` holds every value a move can reach: the
    // highest is `7F + 15 + 12 * (FF + 1)`, the inversions and then the `OP`
    // octave, and the lowest a drop below `00`.
    let mut pitches: ArrayVec<i16, MAX_TONES> = semitones
        .iter()
        .enumerate()
        .filter(|&(position, _)| voicing.keeps(position))
        .map(|(_, &semitone)| i16::from(root.value()) + i16::from(semitone))
        .collect();

    // `tones_needed` admitted at least one tone that survives removal, so
    // there is always a lowest tone to move.
    invert(&mut pitches, inversion);

    if voicing.opens() {
        for pitch in pitches.iter_mut().skip(1).step_by(2) {
            *pitch += OCTAVE;
        }
    }
    // `tones_needed` admitted at least as many tones as the deepest position
    // dropped, so the subtraction cannot pass the first tone.
    let top = pitches.len();
    for &position in voicing.dropped_from_the_top() {
        pitches[top - position] -= OCTAVE;
    }
    pitches.sort_unstable();

    pitches
        .into_iter()
        .map(|tone| {
            note_at(tone)
                .ok_or_else(|| InterpretationError::ChordToneOutOfRange { function, tone }.into())
        })
        .collect()
}

/// Moves the lowest of `pitches`, sorted lowest first and not empty, up an
/// octave `inversion` times, keeping them sorted.
///
/// Only the raised tone is out of place after each move, so it is moved into
/// its place among the others, which stay in order, rather than sorting them
/// all again.
fn invert(pitches: &mut [i16], inversion: u8) {
    for _ in 0..inversion {
        let raised = pitches[0] + OCTAVE;
        pitches[0] = raised;
        let place = pitches[1..].partition_point(|&pitch| pitch < raised);
        pitches[..=place].rotate_left(1);
    }
}

/// Which tones a Voicing keeps, by position counted from zero at the lowest.
enum Kept {
    /// Every tone but these, which are the lowest positions.
    AllBut(&'static [usize]),
    /// Only these, listed lowest first.
    Only(&'static [usize]),
}

/// A Voicing's rules, in the order a Chord Play applies them: the tones it
/// keeps, the tones it raises, and the tones it drops. How many tones it needs
/// is read off those rules, so a new Voicing states each rule once.
impl Voicing {
    const fn kept(self) -> Kept {
        match self {
            Self::Shell => Kept::Only(&[0, 1, 3]),
            Self::Rootless => Kept::AllBut(&[0]),
            Self::Close
            | Self::Open
            | Self::DropTwo
            | Self::DropThree
            | Self::DropFive
            | Self::DropSix => Kept::AllBut(&[]),
        }
    }

    /// Whether the Voicing keeps the tone at `position`, counted from zero.
    fn keeps(self, position: usize) -> bool {
        match self.kept() {
            Kept::AllBut(removed) => !removed.contains(&position),
            Kept::Only(kept) => kept.contains(&position),
        }
    }

    /// Whether the Voicing raises every second kept tone an octave.
    fn opens(self) -> bool {
        self == Self::Open
    }

    /// The positions a drop voicing lowers an octave, counted from the top
    /// from one. The digit in a drop voicing's name is the sum of these.
    const fn dropped_from_the_top(self) -> &'static [usize] {
        match self {
            Self::DropTwo => &[2],
            Self::DropThree => &[3],
            Self::DropFive => &[2, 3],
            Self::DropSix => &[2, 4],
            Self::Close | Self::Open | Self::Shell | Self::Rootless => &[],
        }
    }

    /// How many tones an Interval Set must hold for the Voicing to mean what
    /// it says: every position it names exists, and enough tones survive
    /// removal for each move to have a tone to act on.
    pub(crate) fn tones_needed(self) -> usize {
        let deepest_drop = self.dropped_from_the_top().iter().copied().max();
        let moved = deepest_drop
            .unwrap_or(1)
            .max(if self.opens() { 2 } else { 1 });
        match self.kept() {
            Kept::AllBut(removed) => removed.len() + moved,
            Kept::Only(kept) => kept
                .last()
                .map_or(moved, |&highest| (highest + 1).max(moved)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{ChordTones, MAX_TONES, OCTAVE, invert, raw_chord_play, timed_chord_play};
    use crate::{
        Anchor, ArgumentError, Atom, Error, Function, Interpretation, InterpretationError,
        Interpreter, IntervalSet, Length, MidiChannel, Note, PlayCommand, Tick, TickInputs,
        TypeError, Velocity, Voicing, interpreter::Context,
    };

    /// Evaluates Source text for one Chord Play over Operand Literals.
    fn interpret(source: &str) -> Result<Interpretation, Error> {
        crate::interpret_source(source)
    }

    /// Evaluates a Chord Play over already resolved operands.
    fn evaluate(function: Function, operands: &[Atom]) -> Result<Interpretation, Error> {
        Interpreter::execute_function(
            function,
            operands.iter().copied(),
            TickInputs::new(Tick::ZERO, Anchor::new(0, 0)).into(),
        )
    }

    /// The Play Commands a Chord Play's answer expands to.
    fn commands(answer: Interpretation) -> Vec<PlayCommand> {
        match answer {
            Interpretation::Chord(chord) => chord.commands().collect(),
            other => panic!("{other:?} is not a Chord Play's answer"),
        }
    }

    /// The tones `C4` with `set`, `voicing` and `inversion` answers, as Note
    /// numbers lowest first.
    fn tones(chord: &str) -> Result<Vec<u8>, Error> {
        Ok(commands(interpret(&format!("!:0102{chord}"))?)
            .into_iter()
            .map(|command| match command {
                PlayCommand::Raw { note, .. } => note.value(),
                other => panic!("{other:?} is not a Raw Play Command"),
            })
            .collect())
    }

    fn note(value: u8) -> Note {
        Note::try_from(value).unwrap()
    }

    /// The Raw Play Commands a Chord Play sends on channel 1 at velocity 2.
    fn raw(tones: &[u8]) -> Vec<PlayCommand> {
        tones
            .iter()
            .map(|&tone| PlayCommand::Raw {
                channel: MidiChannel::try_from(1).unwrap(),
                velocity: Velocity::try_from(2).unwrap(),
                note: note(tone),
            })
            .collect()
    }

    /// The Timed Play Commands a Chord Play sends on channel 1 at velocity 2.
    fn timed(tones: &[u8], length: u8) -> Vec<PlayCommand> {
        tones
            .iter()
            .map(|&tone| PlayCommand::Timed {
                channel: MidiChannel::try_from(1).unwrap(),
                velocity: Velocity::try_from(2).unwrap(),
                note: note(tone),
                length: Length::from(length),
            })
            .collect()
    }

    #[test]
    fn a_raw_chord_play_answers_one_raw_play_command_per_tone_lowest_first() {
        // `C4M7` is C4, E4, G4, B4: Notes 3C, 40, 43 and 47.
        assert_eq!(
            commands(interpret("!:0102C4M7CL00").unwrap()),
            raw(&[0x3C, 0x40, 0x43, 0x47])
        );
    }

    #[test]
    fn a_timed_chord_play_answers_one_timed_play_command_per_tone_carrying_its_length() {
        assert_eq!(
            commands(interpret("!#0102C4M_CL0004").unwrap()),
            timed(&[0x3C, 0x40, 0x43], 0x04)
        );
    }

    #[test]
    fn expanding_a_chord_reports_how_many_commands_it_holds() {
        // `Vec::extend` reserves from the exact length, so a Tick grows its
        // Effects once per chord rather than once per tone.
        let Interpretation::Chord(chord) = interpret("!:0102C4chCL00").unwrap() else {
            panic!("a Chord Play answers its tones");
        };
        assert_eq!(chord.commands().len(), 12);
        assert_eq!(chord.commands().size_hint(), (12, Some(12)));
    }

    #[test]
    fn each_operand_reaches_the_role_its_signature_names() {
        // Every operand differs from the others and sits inside the domain of
        // each role it could be transposed into, so only a declaration that
        // kept the roles apart answers exactly these commands.
        let operands = [
            Atom::Number(0x01),
            Atom::Number(0x02),
            Atom::Note(note(0x3C)),
            Atom::IntervalSet(IntervalSet::Major),
            Atom::Voicing(Voicing::Close),
            Atom::Number(0x03),
        ];
        let timed_operands = [
            operands[0],
            operands[1],
            operands[2],
            operands[3],
            operands[4],
            operands[5],
            Atom::Number(0x05),
        ];

        // C4, E4, G4 inverted three times is C5, E5, G5.
        let expected_raw = raw(&[0x48, 0x4C, 0x4F]);
        let expected_timed = timed(&[0x48, 0x4C, 0x4F], 5);

        assert_eq!(
            commands(evaluate(Function::RawChordPlay, &operands).unwrap()),
            expected_raw
        );
        assert_eq!(commands(interpret("!:0102C4M_CL03").unwrap()), expected_raw);
        assert_eq!(
            commands(evaluate(Function::TimedChordPlay, &timed_operands).unwrap()),
            expected_timed
        );
        assert_eq!(
            commands(interpret("!#0102C4M_CL0305").unwrap()),
            expected_timed
        );
    }

    /// A Chord Play body, so the two share one arity claim.
    type Body = fn(&mut Context) -> Result<ChordTones, Error>;

    #[test]
    fn each_chord_play_requires_exactly_its_declared_operands() {
        // A well-typed prefix at every length: what is missing is the count
        // rather than a type, and the body owns that demand.
        let operands = [
            Atom::Number(0x01),
            Atom::Number(0x02),
            Atom::Note(note(0x3C)),
            Atom::IntervalSet(IntervalSet::Major),
            Atom::Voicing(Voicing::Close),
            Atom::Number(0x00),
            Atom::Number(0x04),
        ];
        for (body, expected) in [
            (raw_chord_play as Body, 6_usize),
            (timed_chord_play as Body, 7_usize),
        ] {
            for found in 0..expected {
                let mut ctx = Context::new(
                    TickInputs::new(Tick::ZERO, Anchor::new(0, 0)).into(),
                    expected,
                );
                for operand in operands[..found].iter().rev() {
                    ctx.stack.push(*operand).unwrap();
                }
                let error = body(&mut ctx).unwrap_err();
                assert!(
                    matches!(
                        error,
                        Error::Argument(ArgumentError::Arity { expected: e, found: f })
                            if e == expected && f == found
                    ),
                    "{found} operands gave {error:?}"
                );
            }
        }
    }

    #[test]
    fn channel_velocity_and_length_take_plays_domains_and_a_bad_one_answers_nothing() {
        for channel in 0x10..=u8::MAX {
            assert!(
                matches!(
                    interpret(&format!("!:{channel:02X}7FC4M_CL00")),
                    Err(Error::Interpretation(InterpretationError::MidiChannel(value)))
                        if value == channel
                ),
                "channel {channel:02X}"
            );
            assert!(
                matches!(
                    interpret(&format!("!#{channel:02X}7FC4M_CL0004")),
                    Err(Error::Interpretation(InterpretationError::MidiChannel(value)))
                        if value == channel
                ),
                "channel {channel:02X}"
            );
        }
        for source in ["!:0080C4M_CL00", "!#0080C4M_CL0004"] {
            assert!(
                matches!(
                    interpret(source),
                    Err(Error::Interpretation(InterpretationError::MidiDataByte {
                        role: "velocity",
                        value: 0x80,
                    }))
                ),
                "{source}"
            );
        }
        // Every byte is a length, and it travels unread.
        for length in 0..=u8::MAX {
            let answer = interpret(&format!("!#007FC4M_CL00{length:02X}")).unwrap();
            assert!(commands(answer).iter().all(|command| matches!(
                command,
                PlayCommand::Timed { length: l, .. } if l.value() == length
            )));
        }
    }

    #[test]
    fn a_number_in_a_note_operand_and_a_note_in_a_number_operand_are_refused() {
        let note_c4 = Atom::Note(note(0x3C));
        let set = Atom::IntervalSet(IntervalSet::Major);
        let close = Atom::Voicing(Voicing::Close);
        for (operands, expected) in [
            // A Number where the root stands.
            (
                [
                    Atom::Number(0),
                    Atom::Number(1),
                    Atom::Number(60),
                    set,
                    close,
                    Atom::Number(0),
                ],
                "root",
            ),
            // A Note where a Number stands, in each Number role.
            (
                [
                    note_c4,
                    Atom::Number(1),
                    note_c4,
                    set,
                    close,
                    Atom::Number(0),
                ],
                "channel",
            ),
            (
                [
                    Atom::Number(0),
                    note_c4,
                    note_c4,
                    set,
                    close,
                    Atom::Number(0),
                ],
                "velocity",
            ),
            (
                [
                    Atom::Number(0),
                    Atom::Number(1),
                    note_c4,
                    set,
                    close,
                    note_c4,
                ],
                "inversion",
            ),
        ] {
            assert!(
                matches!(
                    evaluate(Function::RawChordPlay, &operands),
                    Err(Error::Type(_))
                ),
                "{expected}"
            );
        }
        // The Number role only Timed Chord Play declares.
        assert!(matches!(
            evaluate(
                Function::TimedChordPlay,
                &[
                    Atom::Number(0),
                    Atom::Number(1),
                    note_c4,
                    set,
                    close,
                    Atom::Number(0),
                    note_c4,
                ]
            ),
            Err(Error::Type(TypeError::Number(_)))
        ));
    }

    #[test]
    fn a_number_or_note_in_a_name_operand_is_refused_unless_its_cells_spell_a_name() {
        let number = Atom::Number(0x69);
        let c4 = Atom::Note(note(0x3C));
        let root = Atom::Note(note(0x3C));
        let channel = Atom::Number(0);
        let velocity = Atom::Number(1);
        let close = Atom::Voicing(Voicing::Close);
        let major = Atom::IntervalSet(IntervalSet::Major);
        let inversion = Atom::Number(0);
        for mistyped in [number, c4] {
            assert!(matches!(
                evaluate(
                    Function::RawChordPlay,
                    &[channel, velocity, root, mistyped, close, inversion]
                ),
                Err(Error::Type(TypeError::IntervalSet(_)))
            ));
            assert!(matches!(
                evaluate(
                    Function::RawChordPlay,
                    &[channel, velocity, root, major, mistyped, inversion]
                ),
                Err(Error::Type(TypeError::Voicing(_)))
            ));
        }

        // Cells that spell a name are one whichever other reading they have:
        // `69` and `13` are Numbers, and `D2` is a Note and a Number.
        assert_eq!(tones("C469CL00").unwrap(), [0x3C, 0x40, 0x43, 0x45, 0x4A]);
        assert_eq!(
            tones("C413CL00").unwrap(),
            [0x3C, 0x40, 0x43, 0x46, 0x4A, 0x51]
        );
        assert_eq!(tones("C4M7D200").unwrap(), [0x37, 0x3C, 0x40, 0x47]);
    }

    #[test]
    fn the_parser_refuses_a_name_of_the_wrong_literal_type() {
        // `C4` spells a Note and a Number; it is no name, and neither is a
        // Voicing where a set stands or the reverse.
        for source in [
            "!:0064C4C4CL00",
            "!:0064C4CLCL00",
            "!:0064C4M7M700",
            "!:0064C4M7C400",
        ] {
            assert!(interpret(source).is_err(), "{source}");
        }
    }

    #[test]
    fn every_voicing_gives_the_tones_it_names() {
        // C4 M7 is 3C 40 43 47. `OP` raises the 2nd and 4th tones an octave;
        // `D2` lowers the 2nd from the top, `D3` the 3rd, `D5` the 2nd and
        // 3rd, and `D6` the 2nd and 4th; `SH` keeps the 1st, 2nd and 4th; and
        // `RL` removes the 1st.
        for (chord, expected) in [
            ("C4M7CL00", &[0x3C, 0x40, 0x43, 0x47][..]),
            ("C4M7OP00", &[0x3C, 0x43, 0x4C, 0x53]),
            ("C4M7D200", &[0x37, 0x3C, 0x40, 0x47]),
            ("C4M7D300", &[0x34, 0x3C, 0x43, 0x47]),
            ("C4M7D500", &[0x34, 0x37, 0x3C, 0x47]),
            ("C4M7D600", &[0x30, 0x37, 0x40, 0x47]),
            ("C4M7SH00", &[0x3C, 0x40, 0x47]),
            ("C4M7RL00", &[0x40, 0x43, 0x47]),
        ] {
            assert_eq!(tones(chord).unwrap(), expected, "{chord}");
        }
    }

    #[test]
    fn every_voicing_with_inversion_applies_removal_then_inversion_then_movement() {
        // Removal happens before inversion and movement after it. Each row
        // below answers differently from what any other order would answer:
        // `SH01` inverted before removal would keep E4, G4 and C5, and `D201`
        // moved before inversion would answer C4, E4, G4 and B4 again.
        for (chord, expected) in [
            ("C4M7CL01", &[0x40, 0x43, 0x47, 0x48][..]),
            ("C4M7OP01", &[0x40, 0x47, 0x4F, 0x54]),
            ("C4M7D201", &[0x3B, 0x40, 0x43, 0x48]),
            ("C4M7D301", &[0x37, 0x40, 0x47, 0x48]),
            ("C4M7D501", &[0x37, 0x3B, 0x40, 0x48]),
            ("C4M7D601", &[0x34, 0x3B, 0x43, 0x48]),
            ("C4M7SH01", &[0x40, 0x47, 0x48]),
            ("C4M7RL01", &[0x43, 0x47, 0x4C]),
        ] {
            assert_eq!(tones(chord).unwrap(), expected, "{chord}");
        }
    }

    #[test]
    fn inversion_moves_the_lowest_tone_up_an_octave_as_many_times_as_its_number_says() {
        for (chord, expected) in [
            ("C4M7CL02", &[0x43, 0x47, 0x48, 0x4C][..]),
            ("C4M7CL04", &[0x48, 0x4C, 0x4F, 0x53]),
            // A fifth move on four tones takes the C5 the first move made
            // up to C6.
            ("C4M7CL05", &[0x4C, 0x4F, 0x53, 0x54]),
        ] {
            assert_eq!(tones(chord).unwrap(), expected, "{chord}");
        }
    }

    #[test]
    fn too_few_tones_for_a_voicing_diagnoses_the_whole_chord() {
        // A triad is the shortest Interval Set the table holds, so `D6` and
        // `SH`, each needing four tones, are the Voicings a real name can fail.
        for (chord, voicing) in [("C4M_D600", Voicing::DropSix), ("C4M_SH00", Voicing::Shell)] {
            assert!(
                matches!(
                    interpret(&format!("!:0102{chord}")),
                    Err(Error::Interpretation(InterpretationError::TooFewTones {
                        function: Function::RawChordPlay,
                        set: IntervalSet::Major,
                        voicing: v,
                    })) if v == voicing
                ),
                "{chord}"
            );
        }
        // The other Voicings need at most three tones, so a triad satisfies
        // each of them.
        for chord in [
            "C4M_CL00", "C4M_OP00", "C4M_D200", "C4M_D300", "C4M_D500", "C4M_RL00",
        ] {
            assert!(tones(chord).is_ok(), "{chord}");
        }
    }

    #[test]
    fn too_few_tones_names_the_function_the_set_the_voicing_and_its_need() {
        assert_eq!(
            interpret("!:0102C4M_SH00").unwrap_err().to_string(),
            "!: cannot voice Interval Set M_ as SH: the Voicing needs 4 tones and the set has 3, \
             so no tone is sent"
        );
        assert_eq!(
            interpret("!#0102C4s2D60004").unwrap_err().to_string(),
            "!# cannot voice Interval Set s2 as D6: the Voicing needs 4 tones and the set has 3, \
             so no tone is sent"
        );
    }

    #[test]
    fn each_voicing_needs_the_tones_its_rules_act_on() {
        // `OP` raises a second tone and `RL` keeps one past the root it
        // removes; each drop needs its deepest position; `SH` names the 4th.
        for (voicing, needed) in [
            (Voicing::Close, 1),
            (Voicing::Open, 2),
            (Voicing::DropTwo, 2),
            (Voicing::DropThree, 3),
            (Voicing::DropFive, 3),
            (Voicing::DropSix, 4),
            (Voicing::Shell, 4),
            (Voicing::Rootless, 2),
        ] {
            assert_eq!(voicing.tones_needed(), needed, "{voicing:?}");
        }
    }

    #[test]
    fn inverting_matches_raising_the_lowest_tone_and_sorting_every_time() {
        // Every Interval Set over the lowest and highest roots, at every
        // inversion, against the definition: raise the lowest an octave and
        // sort the whole chord again.
        for set in IntervalSet::ALL {
            for root in [0x00_i16, 0x3C, 0x7F] {
                let start: Vec<i16> = set
                    .semitones()
                    .iter()
                    .map(|&semitone| root + i16::from(semitone))
                    .collect();
                let mut expected = start.clone();
                for inversion in 0..=u8::MAX {
                    let mut pitches = start.clone();
                    invert(&mut pitches, inversion);
                    assert_eq!(
                        pitches, expected,
                        "{set:?} on {root:02X}, inversion {inversion}"
                    );
                    expected[0] += OCTAVE;
                    expected.sort_unstable();
                }
            }
        }
    }

    #[test]
    fn a_tone_outside_the_note_range_diagnoses_the_whole_chord() {
        // G9 is the Note 7F, so M_ reaches past it; C/ is 00, so a drop falls
        // below it. Neither sends the tones that do fit. Every tone is moved
        // before any is checked, so `D6` reports its lowest tone, the root
        // dropped an octave, rather than the first one to leave the range.
        for (source, function, tone) in [
            ("!:0064G9M_CL00", Function::RawChordPlay, 131),
            ("!:0064C/M7D200", Function::RawChordPlay, -5),
            ("!:0064C/M7D600", Function::RawChordPlay, -12),
            ("!#0064G9M_CL0004", Function::TimedChordPlay, 131),
        ] {
            assert!(
                matches!(
                    interpret(source),
                    Err(Error::Interpretation(
                        InterpretationError::ChordToneOutOfRange { function: f, tone: t }
                    )) if f == function && t == tone
                ),
                "{source}"
            );
        }
        // Inversion reaches past the range the same way however many times it
        // moves the lowest tone.
        assert!(matches!(
            interpret("!:0064C4M_CLFF"),
            Err(Error::Interpretation(
                InterpretationError::ChordToneOutOfRange { .. }
            ))
        ));
    }

    #[test]
    fn a_tone_outside_the_note_range_is_named_with_its_function_in_decimal() {
        assert_eq!(
            interpret("!:0064G9M_CL00").unwrap_err().to_string(),
            "!: cannot sound chord tone 131, outside the MIDI note range 0–127"
        );
        assert_eq!(
            interpret("!#0064C/M7D60004").unwrap_err().to_string(),
            "!# cannot sound chord tone -12, outside the MIDI note range 0–127"
        );
    }

    #[test]
    fn a_tone_at_the_top_of_the_note_range_is_sent() {
        // C9 M_ is C9, E9 and G9, and G9 is the highest Note, 7F.
        assert_eq!(tones("C9M_CL00").unwrap(), [0x78, 0x7C, 0x7F]);
    }

    #[test]
    fn a_chord_holds_at_most_twelve_tones_and_the_chromatic_scale_fills_them() {
        assert_eq!(MAX_TONES, 12);
        assert_eq!(
            tones("C4chCL00").unwrap(),
            [
                0x3C, 0x3D, 0x3E, 0x3F, 0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47
            ]
        );
    }

    #[test]
    fn velocity_zero_and_length_zero_are_ordinary_tone_commands() {
        // What these mean is the Playback Engine's: a Chord Play carries them
        // on as a Timed Play does, so a body that dropped a tone here would
        // be interpreting them itself.
        for source in ["!#0000C4M_CL0004", "!#007FC4M_CL0000"] {
            assert_eq!(commands(interpret(source).unwrap()).len(), 3, "{source}");
        }
    }
}
