# Musical terminology: decisions so far

Settled with Toby on 13 November 2026, while grilling the audit's renames. Tickets 09–18 implement
this list.

## Interval Set names

- The filler `_` is removed from every name.
- Triads: `M_` → `MA` (major), `m_` → `mi` (minor), `o_` → `di` (diminished), `+_` → `au`
  (augmented).
- The major and natural minor scales take their mode names to free `MA` and `mi`: major scale
  `MA` → `io` (Ionian), natural minor `mi` → `ae` (Aeolian).
- Dominants: `7_` → `07`, `9_` → `09`, beside `13`. `b9` is unchanged.
- Pentatonics: `p5` → `pM`, `p3` → `pm`.
- ADR 0105 is amended to match.

## Voicings

- `D5` → `23` (drop 2&3), `D6` → `24` (drop 2&4). `D2` and `D3` are unchanged.

## Functions and Roles

- Delay `~*` is renamed **Every**; its spelling is unchanged. The research notes' separate `~/`
  Every proposal is dropped. The Sound Module's echo effect keeps the name Delay.
- Interpolation `~>` is renamed **Slew**; its spelling is unchanged.
- Clock, Increment, Random, Track, Push, Halt and Raw Play keep their names and gain musician
  glosses (ticket 06).
- Stack's `degree` Role → `index`, zero-based like Pick's.
- Clock's and Every's `length` Role → `rate`, so `length` only means a note's gate length.
- Up's and Down's `number` Role → `semitones`.

## Built-in Synth

- The acid voice moves from channel `09` to `08`; the 808-style kit takes `09`–`0F`, so MIDI
  channel 10 is a drum channel as in General MIDI.
- The effect controllers move from the Learn Orca (Enfer) layout to General MIDI controllers:
  `MD` CC1, `OF` CC73 (Attack Time), `LN` CC72 (Release Time), `EQ` CC74 (Brightness), `R1` CC91,
  `R2` CC92, `CH` CC93, `DL` CC94. Reverb, chorus and echo become per-channel sends; the others stay
  global.
- The drum map is realigned to General MIDI drum keys, each family with its GM neighbours; every
  other key is silent and the octave-free mapping goes.
- The synth's internal parameter smoother is renamed `Smoothing`.
- Euclidean `~%` is aligned with the published Bjorklund rotations.

## Monophonic Play

- `!%` gains an optional last operand, `legato`, a Number: blank or `00` retriggers (Note Off
  before Note On, as today), anything else sends the new Note On before the old Note Off, so the
  acid voice slides. It needs an operand type whose blank means a fixed value, beside ADR 0108's
  `Optional<O>`; that is a prerequisite ticket with its own ADR.

## Open

- Note spelling: sharps are already lowercase letters; Toby's intent is being clarified.
