# Musical terminology: decisions so far

Settled with Toby on 13 November 2026, while grilling the audit's renames, and revised the same day
while grilling the PR #383 review (ticket 01). Tickets 02–13 implement this list, each with its
own docs.

## Interval Set names

- The filler `_` is removed from every name.
- Triads: `M_` → `MA` (major), `m_` → `mi` (minor), `o_` → `di` (diminished), `+_` → `+5`
  (augmented). Both augmented chords use `+` (`+5`, `+M`), so `a` only means "add" (`a9`).
- Diminished seventh `o7` → `d7`, beside `di` and `h7`, so no name differs from another only by
  `0` against `o` (`07` is the dominant seventh).
- The major and natural minor scales take their mode names to free `MA` and `mi`: major scale
  `MA` → `io` (Ionian), natural minor `mi` → `ae` (Aeolian). `MA` and `mi` change meaning rather
  than disappear, so every existing scale use moves to `io` or `ae`; nothing diagnoses one left
  behind.
- A mode is its first two letters (`io`, `ae`, `do`), except Mixolydian, `mx`.
- Dominants: `7_` → `07`, `9_` → `09`, beside `13`. `b9` is unchanged.
- Pentatonics: `p5` → `pM`, `p3` → `pm`, because `p5` and `p3` read as a power chord and a count
  of tones.
- Accepted near pairs, recorded beside `mM`/`mm` and `pM`/`pm`: `io`/`lo`, `di`/`do`, and the
  Voicings `23`/`24` beside `D2`/`D3`.
- ADR 0105 is amended to match.

## Voicings

- `D5` → `23` (drop 2&3), `D6` → `24` (drop 2&4). `D2` and `D3` are unchanged. The reference
  says the two-Cell limit is why the drop 2&3 and drop 2&4 names carry no `D`.

## Functions and Roles

- Delay `~*` is renamed **Every**; its spelling is unchanged. The research notes' separate `~/`
  Every proposal is dropped. The Sound Module's echo effect keeps the name Delay.
- Interpolation `~>` is renamed **Slew**; its spelling is unchanged.
- Clock, Increment, Random, Track, Push, Halt and Raw Play keep their names and gain musician
  glosses (tickets 05 and 06).
- Stack's `degree` Role → `index`, zero-based like Pick's.
- Clock's and Every's `length` Role → `division`, so `length` only means a note's gate length. It
  counts Ticks per step, so a higher value is slower, as on a clock divider; Orca's `rate` reads
  backwards and is not used. Every keeps its name; ticket 05's gloss notes that Tidal's `every`
  means something else.
- Up's and Down's `number` Role → `semitones`.

## Built-in Synth

- The acid voice moves from channel `09` to `08`; the 808-style kit takes `09`–`0F`, so MIDI
  channel 10 is a drum channel as in General MIDI.
- The effect controllers move from the Learn Orca (Enfer) layout to standard MIDI controllers, and
  every effect is set per channel:
  - `MD` vibrato CC1 (Modulation).
  - `OF` offset CC85, an undefined controller, documented as Orcvs's own: it skips into the sound,
    the reverse of CC73 Attack Time.
  - `LN` length CC75 (Decay Time), which is also the acid voice's decay.
  - `EQ` tilt CC74 (Brightness).
  - `R1` small reverb CC91 and `CH` chorus CC93, the General MIDI reverb and chorus sends.
  - `R2` large reverb CC92 (Effects 2 Depth), Orcvs's own second reverb send.
  - `DL` delay CC94, the delay send as on Roland GS modules: the SC-88 Pro owner's manual lists
    controller 94 as "Effect 4 (Delay Send Level)" in its MIDI implementation appendix.
- On the acid voice's channel its own controllers win: CC74 is its cutoff and CC75 its decay, and
  the tilt and length effects do not apply there.
- Reset All Controllers follows the MIDI Manufacturers Association's RP-015: it clears performance
  controllers and leaves Sound Controllers (CC70–79) and Effect Controllers (CC91–95) alone, so it
  no longer resets the acid voice's sound. Check the wording against the published RP-015 before
  an ADR cites it.
- The drum map is realigned to General MIDI drum keys, each family with its GM neighbours; every
  other key is silent and the octave-free mapping goes.
- The synth's internal parameter smoother is renamed `Smoothing`.

## Euclidean

- `~%` matches Tidal's `bjorklund`, kept as a test-only reference. E(3,4) becomes `xxx.`, so the
  shipped `~%0304` changes; the ADR amendment records that, and that Orca parity is lost.

## Monophonic Play

- `!%` gains an optional last operand, `legato`, a Number: blank or `00` retriggers (Note Off
  before Note On, as today), anything else sends the new Note On before the old Note Off, so the
  acid voice slides. It needs an operand type whose blank means a fixed value, beside ADR 0108's
  `Optional<O>`; that is a prerequisite ticket (12) with its own ADR.

## Notes and numbers in the docs

- Note spelling is documented, not changed: lowercase letters are sharps, and the reference gives
  each flat's sharp spelling (ticket 06).
- Prose names a number by its standard decimal name with the Source hex beside it ("CC74 (`4A`)",
  "velocity 100 (`64`)") for controller numbers, velocities and interval sizes, and the reference
  carries a table of common values. Every ticket that writes docs follows this rule.
- The `C4 + 1 = C5` arithmetic example is replaced by one a musician cannot read as an octave
  jump, and points to Up for transposition (ticket 06).

## Out of scope

- MIDI clock out, swing and the fixed four Ticks per beat are timing features, not terminology.
  `docs/research/timing-operators.md` already notes that triplets do not fit four Ticks per beat.

## Open

- Nothing.
