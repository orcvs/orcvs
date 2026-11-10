//! The two literal types a Chord is written with beside its root Note: the
//! Interval Set and the Voicing.
//!
//! Each name fills both Cells of its pair, so a half-blank pair spells no
//! name and stays partial input as it is everywhere else.

use crate::{
    Error, Function, InterpretationError, Length, MidiChannel, Note, PlayCommand, Spelling,
    Velocity,
};
use arrayvec::ArrayVec;

/// The most tones an Interval Set names: the chromatic scale's twelve. The
/// table below is checked against it at compile time, so every Interval Set's
/// semitones fit a buffer of this many.
pub(crate) const MAX_TONES: usize = 12;

/// Mints [`IntervalSet`] with its complete list, its names and its semitone
/// table from one row each, so a name and its semitones cannot drift apart.
macro_rules! define_interval_sets {
    ($($(#[$doc:meta])* $variant:ident => $name:literal [$($semitone:literal),+]),+ $(,)?) => {
        /// A list of semitones above a root, named by two Cells: a chord or
        /// a scale.
        ///
        /// Chords and scales are one type so that one Function can pick a
        /// tone from either and name what a stack of tones forms.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum IntervalSet {
            $($(#[$doc])* $variant,)+
        }

        impl IntervalSet {
            /// Every Interval Set, in table order.
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            /// The two Cells that name this Interval Set.
            pub const fn spelling(self) -> Spelling {
                match self {
                    $(Self::$variant => const { Spelling::checked($name) },)+
                }
            }

            /// The semitones above the root this Interval Set names, lowest
            /// first. The first is always `0`, and there are at most
            /// twelve. Reading them allocates nothing.
            pub const fn semitones(self) -> &'static [u8] {
                match self {
                    $(Self::$variant => &[$($semitone),+],)+
                }
            }
        }
    };
}

define_interval_sets! {
    /// Major triad.
    MajorTriad => "M_" [0, 4, 7],
    /// Major seventh.
    MajorSeventh => "M7" [0, 4, 7, 11],
    /// Major ninth.
    MajorNinth => "M9" [0, 4, 7, 11, 14],
    /// Major thirteenth: `D` is 13 in hexadecimal.
    MajorThirteenth => "MD" [0, 4, 7, 11, 14, 21],
    /// Major sixth.
    MajorSixth => "M6" [0, 4, 7, 9],
    /// Six-nine.
    SixNine => "69" [0, 4, 7, 9, 14],
    /// Augmented major seventh, with its implied 7 dropped.
    AugmentedMajorSeventh => "+M" [0, 4, 8, 11],
    /// Minor triad.
    MinorTriad => "m_" [0, 3, 7],
    /// Minor seventh.
    MinorSeventh => "m7" [0, 3, 7, 10],
    /// Minor-major seventh, with its implied 7 dropped.
    MinorMajorSeventh => "mM" [0, 3, 7, 11],
    /// Minor sixth.
    MinorSixth => "m6" [0, 3, 7, 9],
    /// Minor ninth.
    MinorNinth => "m9" [0, 3, 7, 10, 14],
    /// Minor eleventh: `B` is 11 in hexadecimal.
    MinorEleventh => "mB" [0, 3, 7, 10, 14, 17],
    /// Minor thirteenth: `D` is 13 in hexadecimal.
    MinorThirteenth => "mD" [0, 3, 7, 10, 14, 21],
    /// Diminished triad.
    DiminishedTriad => "o_" [0, 3, 6],
    /// Diminished seventh.
    DiminishedSeventh => "o7" [0, 3, 6, 9],
    /// Half-diminished seventh.
    HalfDiminishedSeventh => "h7" [0, 3, 6, 10],
    /// Dominant flat ninth, with its implied 7 dropped.
    DominantFlatNinth => "b9" [0, 4, 7, 10, 13],
    /// Dominant seventh.
    DominantSeventh => "7_" [0, 4, 7, 10],
    /// Dominant ninth.
    DominantNinth => "9_" [0, 4, 7, 10, 14],
    /// Dominant thirteenth.
    DominantThirteenth => "13" [0, 4, 7, 10, 14, 21],
    /// Suspended second.
    SuspendedSecond => "s2" [0, 2, 7],
    /// Suspended fourth.
    SuspendedFourth => "s4" [0, 5, 7],
    /// Augmented triad.
    AugmentedTriad => "+_" [0, 4, 8],
    /// Added ninth.
    AddedNinth => "a9" [0, 4, 7, 14],
    /// Major scale.
    Major => "MA" [0, 2, 4, 5, 7, 9, 11],
    /// Natural minor scale.
    Minor => "mi" [0, 2, 3, 5, 7, 8, 10],
    /// Dorian mode.
    Dorian => "do" [0, 2, 3, 5, 7, 9, 10],
    /// Phrygian mode.
    Phrygian => "ph" [0, 1, 3, 5, 7, 8, 10],
    /// Lydian mode.
    Lydian => "ly" [0, 2, 4, 6, 7, 9, 11],
    /// Mixolydian mode.
    Mixolydian => "mx" [0, 2, 4, 5, 7, 9, 10],
    /// Locrian mode.
    Locrian => "lo" [0, 1, 3, 5, 6, 8, 10],
    /// Harmonic minor scale.
    HarmonicMinor => "hm" [0, 2, 3, 5, 7, 8, 11],
    /// Melodic minor scale. Differs from `mM` only by case.
    MelodicMinor => "mm" [0, 2, 3, 5, 7, 9, 11],
    /// Major pentatonic scale.
    MajorPentatonic => "p5" [0, 2, 4, 7, 9],
    /// Minor pentatonic scale.
    MinorPentatonic => "p3" [0, 3, 5, 7, 10],
    /// Blues scale.
    Blues => "bl" [0, 3, 5, 6, 7, 10],
    /// Whole-tone scale.
    WholeTone => "wt" [0, 2, 4, 6, 8, 10],
    /// Chromatic scale.
    Chromatic => "ch" [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
}

impl IntervalSet {
    /// The Interval Set two Cells name, if any.
    pub fn from_spelling(cells: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|set| set.spelling().cells() == cells.as_bytes())
    }

    /// The Interval Set that names exactly `semitones`, if any. No two
    /// Interval Sets name the same list, so the answer is unique.
    pub(crate) const fn from_semitones(semitones: &[u8]) -> Option<Self> {
        let mut index = 0;
        while index < Self::ALL.len() {
            if same_semitones(Self::ALL[index].semitones(), semitones) {
                return Some(Self::ALL[index]);
            }
            index += 1;
        }
        None
    }
}

const fn same_semitones(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}

// Every Interval Set starts at its root, rises strictly, names at most
// `MAX_TONES` semitones, and is the only one naming its list, so naming a list of
// semitones back to an Interval Set is unambiguous.
const _: () = {
    let mut index = 0;
    while index < IntervalSet::ALL.len() {
        let set = IntervalSet::ALL[index];
        let semitones = set.semitones();
        assert!(
            semitones.len() <= MAX_TONES,
            "an Interval Set names at most MAX_TONES semitones"
        );
        assert!(semitones[0] == 0, "an Interval Set starts at its root");
        let mut tone = 1;
        while tone < semitones.len() {
            assert!(
                semitones[tone - 1] < semitones[tone],
                "an Interval Set rises"
            );
            tone += 1;
        }
        assert!(
            matches!(IntervalSet::from_semitones(semitones), Some(found) if found as usize == set as usize),
            "no two Interval Sets name the same semitones",
        );
        index += 1;
    }
};

/// How a Chord's tones are arranged before they sound, named by two Cells.
///
/// Inversion is a Number of its own and not part of the Voicing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Voicing {
    /// `CL`: the tones where the Interval Set puts them.
    Close,
    /// `OP`: every second tone raised an octave.
    Open,
    /// `D2`: the second tone from the top dropped an octave.
    Drop2,
    /// `D3`: the third tone from the top dropped an octave.
    Drop3,
    /// `D5`: the second and third from the top dropped; the digit is the sum
    /// of the positions dropped.
    Drop5,
    /// `D6`: the second and fourth from the top dropped; the digit is the sum
    /// of the positions dropped.
    Drop6,
    /// `SH`: only the 1st, 2nd and 4th tones.
    Shell,
    /// `RL`: every tone but the 1st.
    Rootless,
}

impl Voicing {
    /// Every Voicing.
    pub const ALL: &'static [Self] = &[
        Self::Close,
        Self::Open,
        Self::Drop2,
        Self::Drop3,
        Self::Drop5,
        Self::Drop6,
        Self::Shell,
        Self::Rootless,
    ];

    /// The two Cells that name this Voicing.
    pub const fn spelling(self) -> Spelling {
        match self {
            Self::Close => const { Spelling::checked("CL") },
            Self::Open => const { Spelling::checked("OP") },
            Self::Drop2 => const { Spelling::checked("D2") },
            Self::Drop3 => const { Spelling::checked("D3") },
            Self::Drop5 => const { Spelling::checked("D5") },
            Self::Drop6 => const { Spelling::checked("D6") },
            Self::Shell => const { Spelling::checked("SH") },
            Self::Rootless => const { Spelling::checked("RL") },
        }
    }

    /// The Voicing two Cells name, if any.
    pub fn from_spelling(cells: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|voicing| voicing.spelling().cells() == cells.as_bytes())
    }
}

impl Voicing {
    /// How many tones an Interval Set needs for this Voicing to mean what it
    /// names: a Drop needs the tones it counts down to, a Shell its 4th, and
    /// Open and Rootless a tone beside the one they move or remove.
    pub const fn tones_needed(self) -> usize {
        match self {
            Self::Close => 1,
            Self::Open | Self::Drop2 | Self::Rootless => 2,
            Self::Drop3 | Self::Drop5 => 3,
            Self::Drop6 | Self::Shell => 4,
        }
    }

    /// Whether this Voicing keeps the Interval Set's tone at `position`,
    /// counted from `0` at the root. Only Shell and Rootless remove tones.
    const fn keeps(self, position: usize) -> bool {
        match self {
            Self::Shell => matches!(position, 0 | 1 | 3),
            Self::Rootless => position != 0,
            _ => true,
        }
    }
}

/// The Play Commands one Chord Play performs: one ordinary Raw or Timed Play
/// Command per tone, lowest first, every tone sharing the channel, velocity
/// and length the Chord Play was given.
///
/// Held inline, at most [`MAX_TONES`] tones, so a Turn that answers a chord
/// asks the allocator for nothing. It is built only whole: a chord with a tone
/// outside the Note range, or too few tones for its Voicing, is a diagnostic
/// and never a partial set of commands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChordCommands {
    channel: MidiChannel,
    velocity: Velocity,
    /// `None` for a Raw Chord Play, the length of every tone for a Timed one.
    length: Option<Length>,
    tones: [Note; MAX_TONES],
    count: u8,
}

/// What a Chord Play was written with, once every operand has bound.
pub(crate) struct ChordOperands {
    pub(crate) function: Function,
    pub(crate) channel: MidiChannel,
    pub(crate) velocity: Velocity,
    pub(crate) root: Note,
    pub(crate) set: IntervalSet,
    pub(crate) voicing: Voicing,
    pub(crate) inversion: u8,
    pub(crate) length: Option<Length>,
}

impl ChordCommands {
    /// Builds the chord `operands` name, or the diagnostic that refuses it.
    ///
    /// The tones are built in one fixed order. The Voicing removes tones from
    /// the Interval Set first; the inversion then moves the lowest tone up an
    /// octave, once per count; and the Voicing then moves the tones it names,
    /// counted on the inverted chord. Every step works on semitones above the
    /// root as `i16`, so neither an inversion of `FF` nor a Drop below the
    /// root can overflow before the range is checked.
    pub(crate) fn build(operands: ChordOperands) -> Result<Self, Error> {
        let ChordOperands {
            function,
            channel,
            velocity,
            root,
            set,
            voicing,
            inversion,
            length,
        } = operands;
        let semitones = set.semitones();
        let needed = voicing.tones_needed();
        if semitones.len() < needed {
            return Err(InterpretationError::TooFewTones {
                set,
                voicing,
                needed,
            }
            .into());
        }

        let mut tones: ArrayVec<i16, MAX_TONES> = semitones
            .iter()
            .enumerate()
            .filter(|&(position, _)| voicing.keeps(position))
            .map(|(_, &semitone)| i16::from(semitone))
            .collect();

        // The tones stay sorted, so the lowest is always the first. An
        // unstable sort of at most twelve values is an in-place insertion
        // sort and asks the allocator for nothing.
        for _ in 0..inversion {
            tones[0] += 12;
            tones.sort_unstable();
        }

        // Counted from the bottom for Open and from the top for a Drop, on
        // the tones the inversion left. `tones_needed` guarantees each index.
        let top = tones.len();
        match voicing {
            Voicing::Open => tones.iter_mut().skip(1).step_by(2).for_each(|tone| *tone += 12),
            Voicing::Drop2 => tones[top - 2] -= 12,
            Voicing::Drop3 => tones[top - 3] -= 12,
            Voicing::Drop5 => {
                tones[top - 2] -= 12;
                tones[top - 3] -= 12;
            }
            Voicing::Drop6 => {
                tones[top - 2] -= 12;
                tones[top - 4] -= 12;
            }
            Voicing::Close | Voicing::Shell | Voicing::Rootless => {}
        }
        tones.sort_unstable();

        let mut notes = [Note::LOWEST; MAX_TONES];
        for (slot, &tone) in notes.iter_mut().zip(&tones) {
            let value = i16::from(root.value()) + tone;
            *slot = u8::try_from(value)
                .ok()
                .and_then(|value| Note::try_from(value).ok())
                .ok_or(InterpretationError::ChordToneOutOfRange { function, value })?;
        }

        Ok(Self {
            channel,
            velocity,
            length,
            tones: notes,
            // At most `MAX_TONES`, which fits a byte.
            count: tones.len() as u8,
        })
    }

    /// The tones this chord sounds, lowest first.
    pub fn tones(&self) -> &[Note] {
        &self.tones[..usize::from(self.count)]
    }

    /// One Play Command per tone, lowest first: Raw for a Raw Chord Play and
    /// Timed for a Timed one.
    pub fn commands(&self) -> impl ExactSizeIterator<Item = PlayCommand> + '_ {
        let (channel, velocity, length) = (self.channel, self.velocity, self.length);
        self.tones().iter().map(move |&note| match length {
            None => PlayCommand::Raw {
                channel,
                velocity,
                note,
            },
            Some(length) => PlayCommand::Timed {
                channel,
                velocity,
                note,
                length,
            },
        })
    }
}

#[cfg(test)]
mod test {
    use super::{IntervalSet, Voicing};
    use crate::{Atom, Error, Token, TypeError};

    /// The spec's semitone table, stated here rather than read from the
    /// table under test.
    const SPEC: [(&str, &[u8]); 39] = [
        ("M_", &[0, 4, 7]),
        ("M7", &[0, 4, 7, 11]),
        ("M9", &[0, 4, 7, 11, 14]),
        ("MD", &[0, 4, 7, 11, 14, 21]),
        ("M6", &[0, 4, 7, 9]),
        ("69", &[0, 4, 7, 9, 14]),
        ("+M", &[0, 4, 8, 11]),
        ("m_", &[0, 3, 7]),
        ("m7", &[0, 3, 7, 10]),
        ("mM", &[0, 3, 7, 11]),
        ("m6", &[0, 3, 7, 9]),
        ("m9", &[0, 3, 7, 10, 14]),
        ("mB", &[0, 3, 7, 10, 14, 17]),
        ("mD", &[0, 3, 7, 10, 14, 21]),
        ("o_", &[0, 3, 6]),
        ("o7", &[0, 3, 6, 9]),
        ("h7", &[0, 3, 6, 10]),
        ("b9", &[0, 4, 7, 10, 13]),
        ("7_", &[0, 4, 7, 10]),
        ("9_", &[0, 4, 7, 10, 14]),
        ("13", &[0, 4, 7, 10, 14, 21]),
        ("s2", &[0, 2, 7]),
        ("s4", &[0, 5, 7]),
        ("+_", &[0, 4, 8]),
        ("a9", &[0, 4, 7, 14]),
        ("MA", &[0, 2, 4, 5, 7, 9, 11]),
        ("mi", &[0, 2, 3, 5, 7, 8, 10]),
        ("do", &[0, 2, 3, 5, 7, 9, 10]),
        ("ph", &[0, 1, 3, 5, 7, 8, 10]),
        ("ly", &[0, 2, 4, 6, 7, 9, 11]),
        ("mx", &[0, 2, 4, 5, 7, 9, 10]),
        ("lo", &[0, 1, 3, 5, 6, 8, 10]),
        ("hm", &[0, 2, 3, 5, 7, 8, 11]),
        ("mm", &[0, 2, 3, 5, 7, 9, 11]),
        ("p5", &[0, 2, 4, 7, 9]),
        ("p3", &[0, 3, 5, 7, 10]),
        ("bl", &[0, 3, 5, 6, 7, 10]),
        ("wt", &[0, 2, 4, 6, 8, 10]),
        ("ch", &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]),
    ];

    const VOICINGS: [(&str, Voicing); 8] = [
        ("CL", Voicing::Close),
        ("OP", Voicing::Open),
        ("D2", Voicing::Drop2),
        ("D3", Voicing::Drop3),
        ("D5", Voicing::Drop5),
        ("D6", Voicing::Drop6),
        ("SH", Voicing::Shell),
        ("RL", Voicing::Rootless),
    ];

    fn decoded_set(cells: &str) -> &'static [u8] {
        match Token::IntervalSet.decode(cells) {
            Ok(Atom::IntervalSet(set)) => set.semitones(),
            other => panic!("{cells:?} decoded to {other:?}"),
        }
    }

    #[test]
    fn every_interval_set_name_decodes_to_the_semitones_the_spec_states() {
        for (name, semitones) in SPEC {
            assert_eq!(decoded_set(name), semitones, "{name}");
        }
        assert_eq!(IntervalSet::ALL.len(), SPEC.len());
    }

    #[test]
    fn every_voicing_name_decodes() {
        for (name, voicing) in VOICINGS {
            assert!(
                matches!(Token::Voicing.decode(name), Ok(Atom::Voicing(found)) if found == voicing),
                "{name}"
            );
        }
        assert_eq!(Voicing::ALL.len(), VOICINGS.len());
    }

    #[test]
    fn a_decoded_name_renders_back_as_the_cells_that_spelled_it() {
        for (name, _) in SPEC {
            assert_eq!(Token::IntervalSet.decode(name).unwrap().to_string(), name);
        }
        for (name, _) in VOICINGS {
            assert_eq!(Token::Voicing.decode(name).unwrap().to_string(), name);
        }
    }

    #[test]
    fn unknown_and_half_blank_names_are_refused() {
        for cells in ["M ", " 7", "M", "  ", "MX", "m8", "C4", "00", "**", "M7 "] {
            assert!(
                matches!(
                    Token::IntervalSet.decode(cells),
                    Err(Error::Type(TypeError::IntervalSet(found))) if found == cells
                ),
                "{cells:?}"
            );
        }
        for cells in ["C ", " L", "C", "  ", "cl", "D4", "D1", "M7", "**", "CLL"] {
            assert!(
                matches!(
                    Token::Voicing.decode(cells),
                    Err(Error::Type(TypeError::Voicing(found))) if found == cells
                ),
                "{cells:?}"
            );
        }
    }

    #[test]
    fn names_that_also_spell_a_number_or_note_are_read_by_the_receiving_operand() {
        // An Operand Literal has no type outside its operand position.
        assert_eq!(Token::Number.decode("69").unwrap(), Atom::Number(0x69));
        assert_eq!(decoded_set("69"), [0, 4, 7, 9, 14]);
        assert_eq!(Token::Number.decode("13").unwrap(), Atom::Number(0x13));
        assert_eq!(decoded_set("13"), [0, 4, 7, 10, 14, 21]);
        assert_eq!(
            Token::Note.decode("D2").unwrap(),
            Atom::Note(crate::Note::try_from(38).unwrap())
        );
        assert_eq!(
            Token::Voicing.decode("D2").unwrap(),
            Atom::Voicing(Voicing::Drop2)
        );
    }

    #[test]
    fn an_interval_set_and_a_voicing_refuse_each_others_names() {
        for (name, _) in VOICINGS {
            assert!(Token::IntervalSet.decode(name).is_err(), "{name}");
        }
        for (name, _) in SPEC {
            assert!(Token::Voicing.decode(name).is_err(), "{name}");
        }
    }

    #[test]
    fn a_list_of_semitones_names_the_interval_set_that_forms_it() {
        for (name, semitones) in SPEC {
            assert_eq!(
                IntervalSet::from_semitones(semitones).map(|set| set.spelling().to_string()),
                Some(name.to_owned()),
            );
        }
        // C major pentatonic stacked from C forms C, E and A, which no
        // Interval Set names.
        assert_eq!(IntervalSet::from_semitones(&[0, 4, 9]), None);
        assert_eq!(IntervalSet::from_semitones(&[]), None);
        assert_eq!(
            IntervalSet::from_semitones(&[0, 4, 7, 11, 14, 21, 24]),
            None
        );
    }
}
