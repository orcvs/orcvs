# Musical terminology

**Status:** ready-for-agent

Orcvs's names, numbering and Built-in Synth behaviour are aligned with what musicians already know
from lead sheets, DAWs, General MIDI and live-coding tools. `audit.md` records the mismatches,
`decisions.md` the settled answers, and tickets 02–13 the work, each a vertical slice that
carries its own code, tests and docs. This spec is the whole effort in
one place; where it and `decisions.md` differ, `decisions.md` wins.

## Problem Statement

A musician who comes to Orcvs with ordinary musical knowledge is misled by it. Chord names carry a
filler character and read like other chords (`07` beside `o7`), and `MA` means a scale rather than
a major triad. Drop voicings are named by a digit sum nobody uses. A Function called Delay delays
nothing, a period is called a `length` or a `rate` though a higher value is slower, and an
Interpolation is really a slew. Stack's scale positions count from zero but are called degrees.
The Built-in Synth plays the acid bassline on MIDI channel 10, where every General MIDI user puts
drums, and maps its effects onto controllers 1–8, where a DAW's volume, balance and data-entry
messages wash reverb and EQ across every channel. Its drums ignore General MIDI keys, its Euclidean
patterns are rotated against Tidal's, Monophonic Play cannot play legato, and Reset All Controllers
wipes the acid voice's sound. The docs mix hex and decimal in one sentence, never say a Tick is a
sixteenth note, and teach `C4 + 1 = C5` as their headline arithmetic.

## Solution

Names say what musicians call the thing, numbers follow the MIDI and General MIDI conventions a
musician's other gear follows, and the docs explain the rest in musician terms:

- Interval Sets drop the filler and take chord-quality and mode names; the drop voicings take their
  standard positions; Delay becomes Every and Interpolation becomes Slew; Roles are named for what
  they hold (`division`, `index`, `semitones`).
- The Built-in Synth puts drums on MIDI channel 10, sets every effect per channel on standard MIDI
  controllers, maps its kit to General MIDI drum keys, and follows MIDI's recommended response to
  Reset All Controllers.
- Euclidean patterns match Tidal's; Monophonic Play gains a `legato` operand.
- The docs state the Tick, the channel numbering, Note spelling and the Built-in Synth's controller
  map, gloss every timing Function, and give each well-known number its decimal name beside the
  hex a musician types.

The language is pre-release, so renamed names are removed outright, with no aliases.

## User Stories

1. As a musician, I want a major triad to be `MA` and a minor triad `mi`, so that the names match
   the jazz MA/mi convention I read on lead sheets.
2. As a musician, I want the diminished triad `di` and the diminished seventh `d7`, so that both
   diminished chords share a letter and neither is a glyph away from the dominant seventh `07`.
3. As a musician, I want both augmented chords written with `+` (`+5`, `+M`), so that `a` only ever
   means "add", as in `a9`.
4. As a musician, I want the dominant seventh and ninth to be `07` and `09`, beside `13`, so that
   dominants read as the bare numbers a lead sheet uses.
5. As a musician, I want the major and natural minor scales named `io` (Ionian) and `ae` (Aeolian),
   so that they sit with the other mode names.
6. As a musician, I want the pentatonics named `pM` and `pm`, so that I don't read `p5` as a power
   chord.
7. As a musician with old Source, I want every scale use of `MA` and `mi` in shipped examples, the
   reference and tests moved to `io` and `ae`, so that nothing the project ships silently plays a
   triad where it meant a scale.
8. As a musician, I want the reference to describe every Interval Set in chord-quality terms, with
   how a lead sheet writes the chord beside its name, so that I can find the chord I'm thinking of.
9. As a musician, I want the reference to say which pairs differ only by case or one letter (`mM`/
   `mm`, `pM`/`pm`, `io`/`lo`, `di`/`do`), so that I watch for them.
10. As a musician, I want drop 2&3 and drop 2&4 named `23` and `24`, so that the name lists the
    voices dropped, as the jazz texts do.
11. As a musician, I want the reference to say the two-Cell limit is why `23` and `24` carry no `D`,
    so that the family's two spellings make sense.
12. As a musician, I want the reference to give each Voicing its standard name and note what `SH`
    and `RL` keep, so that I can match them to the voicings I already play.
13. As a musician, I want the clock divider called Every, so that I don't confuse it with the
    Sound Module's echo effect, which keeps the name Delay.
14. As a musician, I want the slew limiter called Slew, so that its name says what it does.
15. As a musician, I want Clock's and Every's Ticks-per-step operand called `division`, so that a
    higher value being slower reads as it does on a clock divider.
16. As a musician, I want `length` to mean only a note's gate length, so that one word never means
    three things.
17. As a musician, I want Stack's position operand called `index`, like Pick's, and the reference to
    give the one-based degree beside it (index `04` of C major is G, the 5th, V), so that I can
    count either way.
18. As a musician, I want Up's and Down's operand called `semitones`, so that I know the unit.
19. As a musician, I want diagnostics to use the same Role names as the reference, so that an error
    names the operand I wrote.
20. As a musician, I want the reference and glossary to state that a Tick is a sixteenth note, four
    to a beat at the displayed BPM, so that I can count bars.
21. As a musician, I want every timing Function glossed in musician terms (clock divider, slew
    limiter, wrapping counter, reproducible random value per Tick and Position), so that I recognise
    each from other tools.
22. As a Tidal user, I want the reference to note that Orcvs's Every fires a pulse, unlike Tidal's
    `every`, so that I'm not misled.
23. As a Tidal user, I want `~% hits steps` to give the same pattern as Tidal's Euclidean rhythms,
    so that patterns I copy sound the same.
24. As a musician, I want zero hits to be silence and zero steps to be refused, so that the edge
    cases behave predictably.
25. As a musician, I want the reference to explain that channels `00`–`0F` are MIDI channels 1–16,
    so that I can match them to my DAW.
26. As a General MIDI user, I want MIDI channel 10 (`09`) to play drums on the Built-in Synth, so
    that a drum pattern I send there plays drums.
27. As a musician, I want the acid voice on `08` and the kit on `09`–`0F`, so that the bassline no
    longer sits on the drum channel.
28. As a General MIDI user, I want the kit mapped to General MIDI drum keys, kick on 36, snare on
    38, closed hat on 42 and so on, with other keys silent, so that a GM drum part plays the right
    instruments.
29. As a General MIDI user, I want reverb on CC91 and chorus on CC93, so that the sends I know work.
30. As a musician, I want every effect set per channel, so that turning one channel's controls does
    not change every other channel.
31. As a musician, I want the vibrato on CC1, the tilt on CC74 (Brightness), the length on CC75
    (Decay Time), the large reverb on CC92, the delay on CC94 as on Roland GS modules, and the offset
    on the undefined CC85, so that each controller either means what MIDI says or claims no standard
    meaning.
32. As a musician, I want CC74 and CC75 on the acid voice's channel to be its cutoff and decay, so
    that brightness and decay mean the same thing there as everywhere else.
33. As a musician, I want Reset All Controllers to clear performance controllers and leave the sound
    controllers and effect sends alone, as MIDI's recommended practice says, so that a reset does not
    wipe the sound I set up.
34. As a musician, I want the Sound Module to show and send each effect on the channel it changes,
    so that what I turn there matches what Source does.
35. As a musician, I want the Knobs example to set each effect on the channel it should change, so
    that the shipped example still shapes its bassline.
36. As a musician, I want the reference to list the Built-in Synth's controllers with their MIDI
    names, its drum keys and which MIDI messages it acts on, so that I can drive it from memory.
37. As a musician, I want Monophonic Play to take an optional `legato` operand, so that I can play
    legato and slide on a mono synth.
38. As a musician, I want a blank or `00` `legato` to retrigger as today, so that existing Source
    behaves the same.
39. As a musician, I want a legato note to send its Note On before the previous Note Off, so that the
    acid voice and external mono synths slide.
40. As a musician, I want a legato note at the same pitch to keep sounding to its new end with no
    Note Off in between, so that a repeated legato note ties rather than cuts out.
41. As a musician, I want the reference to say an external synth slides only with its portamento
    on, so that I know why one might not.
42. As a language designer, I want an operand type whose blank means a fixed value, recorded in an
    ADR beside ADR 0108's optional operands, so that `legato` and later operands can default.
43. As a musician, I want the reference to explain Note spelling: scientific pitch notation with
    middle C as `C4`, lowercase letters as sharps, and each flat's sharp spelling, so that I can
    write any note and know my DAW may call `C4` C3.
44. As a musician, I want prose to name controller numbers, velocities and interval sizes in decimal
    with the Source hex beside them, and the reference to carry a table of common values, so that I
    can move between my DAW and Source.
45. As a musician, I want the headline arithmetic example to be one I can't read as an octave jump,
    and to point me to Up for transposition, so that I learn the right tool.
46. As a contributor, I want the glossary and the ADRs to state each new name and behaviour in the
    present tense, so that they remain the sources of truth.
47. As an agent picking up a ticket, I want each ticket to name every place its change reaches and
    every gate that covers it, so that the ticket closes with CI green.

## Implementation Decisions

- **Interval Sets.** The filler is removed. Triads `MA mi di +5`; diminished seventh `d7`; scales
  `io ae` and `pM pm`; dominants `07 09` beside `13`; `b9`, `+M`, `h7` and the other names are
  unchanged. The uniqueness check across Interval Sets, Voicings and Functions still holds. ADR 0105
  is amended. `MA` and `mi` change meaning, so migrating every scale use is part of the rename, not
  a follow-up: nothing will diagnose one left behind.
- **Voicings.** `D5` → `23`, `D6` → `24`; the Rust variants are named for the dropped positions and
  the digit-sum explanation goes.
- **Functions and Roles.** Delay `~*` → Every, Interpolation `~>` → Slew, spellings unchanged;
  Clock's and Every's `length` → `division`; Stack's `degree` → `index`; Up's and Down's `number` →
  `semitones`. Info, the reference, the console's Function Reference, diagnostics, the glossary and
  the examples change together. Slew's `amount` is reviewed in ticket 05.
- **Built-in Synth channels.** The acid voice moves to `08`, the kit to `09`–`0F`.
- **Built-in Synth controllers.** Every effect is per channel: `MD` CC1, `OF` CC85, `LN` CC75, `EQ`
  CC74, `R1` CC91, `R2` CC92, `CH` CC93, `DL` CC94. On the acid voice's channel its own controllers
  win (CC74 cutoff, CC75 decay). The Sound Module shows and sends effects per channel. The synth's
  parameter smoother is renamed `Smoothing`. ADRs 0087, 0088 and 0099 are amended.
- **Reset All Controllers** follows RP-015: it clears performance controllers and leaves Sound
  Controllers (CC70–79) and Effect Controllers (CC91–95) alone. Check RP-015's published wording
  before an ADR cites it.
- **Drum keys.** The kit answers General MIDI drum keys, each family with its GM neighbours; every
  other key is silent, and the octave-free mapping goes.
- **Euclidean.** `~%` matches Tidal's `bjorklund` for 1 ≤ hits ≤ steps ≤ 32. E(3,4) becomes `xxx.`,
  so shipped `~%0304` changes. ADR 0012 is amended to record the change and the loss of Orca parity.
- **Blank defaults and legato.** A new operand type gives blank a fixed value, with its own ADR
  grounded in ADR 0108's deferral of operand defaults. Monophonic Play gains a last `legato` Number
  of that type: blank or `00` retriggers; anything else sends the new Note On before the old Note
  Off, and a same-pitch legato note sends no Note Off and runs to its new expiry. Playback's
  scheduler owns the ordering. ADRs 0016 and 0099 are amended.
- **Docs.** The reference states the Tick, channel numbering, Note spelling and the Built-in Synth's
  controllers, keys and MIDI messages; glosses the timing Functions; and follows the decimal-beside-
  hex rule. Each slice writes the docs for its own change.
- **Order.** The renames (02–05), Euclidean (07), the channel move (08) and the blank-default
  operand type (12) can start at once. The notation docs (06) follow Every and Slew (05); the drum
  keys (09) and per-channel effects (10) follow the channel move; Reset All Controllers (11)
  follows the effects; legato (13) follows the operand type. 02 and 03 both amend ADR 0105 and
  the glossary, so whichever lands second rebases.

## Testing Decisions

- A good test drives Orcvs the way a musician does and checks what comes out: Source in, MIDI
  messages and Bangs out at a given Tick; MIDI bytes in, synth state or sound out. No test reads
  private state or depends on how a Function is implemented.
- **Seam 1: the Function reference's runnable examples.** Each renamed name, Role and behaviour gets
  an "`orcvs tick N plays K`" example in the reference, run by the existing reference-doc test in
  `orcvs`: the Interval Sets and Voicings, Every, Slew, `division`, `index`, `semitones`, Euclidean
  patterns and legato. The shipped-example tests cover the examples the same way.
- **Seam 2: the Built-in Synth's MIDI input.** The synth crate's existing byte-level tests cover the
  channel layout, the controller map and per-channel sends, the acid voice's controllers winning on
  `08`, Reset All Controllers, and the drum keys.
- The Euclidean test compares every 1 ≤ hits ≤ steps ≤ 32 against a Bjorklund reference written in
  test code only, never in shipped code.
- Legato tests cover a retrigger, a slide, and two consecutive legato notes at the same pitch.
- The site's Playwright specs remain the check on the Sound Module.
- The `console` gates run wherever the console compiles what a ticket changes: the examples or the
  Function Reference asset.
- Prior art: the reference-doc test in `orcvs`, the shipped-example playback tests, the synth's
  byte-level tests, and the site's Sound Module spec.

## Out of Scope

- MIDI clock out, swing, triplets, and changing the four Ticks per beat; these are timing features.
- A flat spelling for Notes.
- New chords, such as 7sus4, 7♯9 and 7♭5.
- Aliases for old names.
- OSC's "type tag string" wording, and the tone voices' `Shape` beside the acid voice's
  `waveform`.
- Correcting `audit.md`: it is a record as of `97fb810`, and this spec supersedes it.
- Program Change, Volume, Pan and Pitch Bend handling in the Built-in Synth, beyond documenting
  what it ignores.

## Further Notes

- `audit.md`'s citations are as of `97fb810`; re-locate each one in the current tree.
- The decisions were settled with Toby on 13 November 2026, in two grilling rounds; the second
  answered the review of PR #383 and is recorded in ticket 01.
