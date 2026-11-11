# Chords and patterns: prior art and proposals

2026-10-30. A companion to [timing-operators.md](timing-operators.md). This is research and design
only, and nothing in it is decided. An accepted proposal goes through the same steps as the timing
proposals: a ticket, a prototype on real Grid Cells, an ADR, then a test-first implementation.

## Summary

- **orcvs-ts already had chords.** `chord('C4:m7')` had 21 qualities, `arp` cycled a chord's
  tones, and `transpose` shifted them. It also had the pattern pieces the timing doc proposes:
  Euclidean with rotation, an inverted pulse, frame windows, nested patterns and a ping-pong
  sequence.
- **A real chord notation is possible (added 2026-10-31).** Use lead-sheet chord symbols such as
  `C4m7`, `G37` and `F4M7/A` in a Payload closed by `!`, the same rule `!u` and `!o` use. A
  Payload holding several chords is a progression. See "A chord notation" below.
- **Orcvs has no Sequence value any more (ADR 0063), so a chord can't be a value.** The fit is one
  Note per Function. A **Chord Tone `.c root quality tone`** answers one tone of a chord, and a
  **Scale Degree `.s root scale degree`** answers one degree of a scale.
- **Timing drives the `tone` and `degree` operands.** Clock gives an up arpeggio, Bounce
  (proposed) gives up-down, Random gives a random arpeggio, and Track gives an arbitrary order.
  Playing the tone indexes from `1` gives a first-inversion triad, so voicings need no extra
  operand. A chord progression is a Track of roots stepped by Every or Clock at bar rate.
- **Playing a whole chord at once** is three Timed Plays with tones `00`, `01` and `02`. A **Chord
  Play `!#`** is optional sugar for that. A **strum** needs the in-Tick scheduling that the timing
  doc's Ratchet needs.

## Prior art in the orcvs org

### orcvs-ts (archived April 2023)

Chords (`src/note.ts`):

- `chord('C4:m7', ...options)` returned the root plus intervals from `ChordIntervalsTable`. The
  qualities were `M M7 M9 M13 M6 M69 M7b6 m m7 mM7 m6 m9 m11 m13 o o7 m7b5 7 7b9 9 13`. `$`
  stood for a sharp in names (`C$4`).
- `arp('C4:M7')` returned a function that stepped through the chord's tones, one per call.
- `transpose(notes, semitones)` shifted a note or a chord, by an octave by default.
- The options `{d, a, r}` (duration, attack, release) could themselves be sequences. They were
  read with `seqL` across the chord's tones, so each tone got the next duration, attack or release
  in a ramp.
- A second table, `ChordTable`, spelled each quality as a stack of intervals (`P1 M3 m3 ...`), and
  `getIntervals` turned a stack into semitones from the root.

Patterns and timing (`src/sequence.ts`, `src/algo.ts`, `src/pulsar.ts`, `src/library.ts`):

- `seq` (cycle), `seqL` (lerp), `seqR` (random), and `wave` (ping-pong, the same idea as the
  proposed Bounce `~^`).
- `euclid(steps, beats, rotate)` had a rotation operand, and `rotate`, `zip`, `flipPulse` (Not)
  and `pulseOnBeat` worked on pulses.
- A pulse string such as `'▮▯▮▮'` was a step pattern.
- `f:n` fired every n frames, and `f:from:to` was a window of frames, which is the proposed
  During.
- **`t:from:to` was the same window in seconds**, converted to frames at the current BPM
  (`timeToFrame`). So seconds were tried once, and they were tempo-dependent at evaluation time.
- `ptn(...)` nested inside a pulsar only ticked when its parent matched. An inner pattern therefore
  stepped once per *parent pulse*, not once per frame, which is the proposed Count `*+` idea.
- The clock ran at 4 frames per beat (`framesPerBeat`), and `framesPerPhrase` was a settable phrase
  length.

### orcvs-code (archived March 2023)

A VS Code language-server scaffold for `.orcvs` files, with `ptn` and `at` snippets. It has no
chord or timing logic. The `at` snippet shows that "at" was already part of the vocabulary.

## Prior art elsewhere

- **TidalCycles**: chord names in mini-notation (`"c'maj e'min7"`), with modifiers for inversion
  (`'i`, `'ii`), open voicing (`'o`), drop voicings (`'d1`) and number of notes (`'5`). `arp` takes
  modes (`up down updown downup converge diverge pinkyup thumbup`), and `scale "major" "0 .. 7"`
  maps degrees to notes.
- **Strudel**: `chord("<C^7 Dm7 G7>").voicing()` uses voicing dictionaries and voice leading, and
  `.arp(...)` picks tones by index pattern.
- **Sonic Pi**: `chord(:e3, :minor, invert: 1, num_octaves: 2)`, `chord_degree(:ii, :c4, :major)`,
  `scale(:c4, :dorian)`, and `.tick` to walk a chord as a ring.
- **Ableton MIDI effects**: Chord, Arpeggiator (Up, Down, UpDown, Converge, Diverge, Pinky, Thumb,
  Random, and Chord Trigger, with rate, gate, octaves and retrigger), and Scale.
- **Bitwig Note FX**: Chord, Arpeggiator (pattern, rate, octaves, gate, per-step velocity),
  Quantize and Harmonize.
- **Orca**: no chords. Players stack several `:` operators, or step through tones with `T` track.

The common thread: a chord is **a root, a quality and a voicing**. An arpeggio is **an index into
the chord that moves in time**. A scale degree is the same idea applied to a scale.

## A chord notation (added 2026-10-31)

Two Cells is the convention for a spelling and an operand, not a limit. Three things in Orcvs
already claim a run of Cells whose length isn't fixed:

- a Comment runs from `||` to the next `||` (ADR 0074);
- the Payload of `!u` and `!o` runs to the next `!` (ADR 0085);
- Track's items run for as many pairs as its count operand says (ADR 0063, ADR 0067).

The Payload is the best fit for chords. The Parser finds its closer when it parses the Source, so
the claim and the schedule stay fixed for the whole Tick. Its Cells are ordinary Source, so a Write
can rewrite a chord mid-performance. The Function reads the Cells at its Turn, as `!o` parses its
OSC words. That means a chord can be written in **lead-sheet notation** without inventing a new
kind of value. This section proposes that notation, and it replaces the two-Cell quality table
below as the preferred design.

### Chord symbols

A chord symbol is one word in a Payload: a **root**, then an optional **quality**, then optional
**modifiers**.

- **Root**: an Orcvs Note, two Cells exactly as everywhere else. `C4` is C4, and `c4` is C♯4. The
  octave is part of the root, so the rest of the word can start with a digit: `G37` is G3 with the
  quality `7`.
- **Quality**: lead-sheet spelling, case-sensitive the way lead sheets are. Lowercase `m` is minor
  and uppercase `M` is major.

| Quality | Chord | Semitones from the root |
|---|---|---|
| none | major | 0 4 7 |
| `m` | minor | 0 3 7 |
| `o` or `dim` | diminished | 0 3 6 |
| `+` or `aug` | augmented | 0 4 8 |
| `sus2`, `sus4` | suspended | 0 2 7, 0 5 7 |
| `6`, `m6`, `69` | sixths | 0 4 7 9, 0 3 7 9, 0 4 7 9 14 |
| `7` | dominant 7th | 0 4 7 10 |
| `M7` | major 7th | 0 4 7 11 |
| `m7` | minor 7th | 0 3 7 10 |
| `mM7` | minor-major 7th | 0 3 7 11 |
| `o7` | diminished 7th | 0 3 6 9 |
| `m7b5` or `h7` | half-diminished | 0 3 6 10 |
| `7b9` | dominant flat 9 | 0 4 7 10 13 |
| `9`, `M9`, `m9` | 9ths | add 14 to the 7th chord |
| `11`, `m11`, `13`, `M13`, `m13` | extensions | as in orcvs-ts's table |
| `add9` | added 9th | 0 4 7 14 |

  These are orcvs-ts's `ChordIntervalsTable` qualities, plus sus, aug and add9, in the spellings
  musicians already read.
- **Modifiers**:
  - `^n` is the n-th inversion: `C4M7^1` puts E at the bottom. This is Tidal's `'i`, `'ii`.
  - `/X` is a slash bass, the Note letter `X` in the octave below the root: `F4M7/A`.
  - `_n` limits the chord to its first n tones, or stretches it past its size by repeating the
    tones an octave up: `C4m_5`. This is Tidal's `'5` and Sonic Pi's `num_octaves`.

  Start with inversion and slash bass. Leave open voicings, drop voicings and voice leading
  (Strudel `.voicing()`) until a real Grid needs them.

A word the notation can't read diagnoses with the word's position, the same way an `!o` argument
that doesn't fit its type does. A tone above `7F` diagnoses as `.^` does.

### A Payload of chords is a progression

The Payload can hold several chord symbols separated by spaces. An `index` operand picks one, at
`index % count`, the way Track picks a pair, and a Clock in `index` steps through the progression.

#### `!#` Chord Play: `!# channel velocity length index`, then a Payload of chords

When a Bang reaches it, `!#` plays every tone of the chosen chord as a Timed Play, with Note Offs
`length` Ticks later. For example, `!#006410~.1004C4M7 A3m7 D4m7 G37!` plays a I-vi-ii-V loop,
one chord per bar.

#### `.c` Chord Tone: `.c index tone`, then a Payload of chords

Answers one Note: tone `tone % size` of the chosen chord, an octave up for each time `tone` wraps
past the chord's size.

- `.c00~.0104C4m7!` is an up arpeggio of Cm7.
- `.c~.4004~^0103C4M7 A3m7 D4m7 G37!` arpeggiates up and down through the progression, changing
  chord every four bars.

Nest `.c` in any Play's note operand. Its Return is one Note, so ADR 0061 is unchanged.

### What this changes in the language

- **One output per Expression.** CONTEXT.md says each Expression performs at most one output, and
  that "a chord is several Terminal Output roots that one Bang activates". `!#` would be the first
  Expression to send several Play Commands. That needs an ADR. The alternative is to drop `!#` and
  keep chords as several Plays, each with a `.c` in its note operand.
- **A Payload on a value Function.** `.c` would be the first value Function with a Payload. The
  Payload rules of ADR 0085 carry over unchanged: the extent is fixed at parse, a Write lands in
  the same Tick, and a written `!` closes the Payload only from the next parse.
- **Closer.** I suggest `!` for both Functions, so there's one Payload rule. `!` is the
  terminal-output character, which reads oddly after a value Function. The prototype should show
  both `!` and a different closer.

### The alternatives

- **(A) Payload chord symbols, as above.** This is my recommendation. It reads as lead-sheet
  notation, holds a progression in one place, can be rewritten live by a Write, and reuses a parse
  rule that already exists.
- **(B) A fixed four-Cell chord: a Note plus a two-Cell quality.** This is the `.c root quality
  tone` proposal below. It needs no new parse rule, but it can't spell `m7b5`, `add9`, inversions
  or slash chords without making up abbreviations.
- **(C) A bracketed chord literal as a new Language Unit**, for example `[C4m7]`, accepted in any
  Note operand. This is the most general, since every Play would take a chord. But it needs
  variable-width operand claims, which ADR 0063 retired, and it changes every Play's output rule
  at once.

## Proposals (two-Cell version)

These follow the timing doc's conventions: two-Cell operands, value Functions in `.`, effects in
`!`.

### `.c` Chord Tone: `.c root quality tone`

Answers the Note of tone `tone` (a Number, counted from `00`) of the chord on `root` (a Note).
Indexes past the last tone wrap up an octave. For a triad, `03` is the root an octave up and `04`
is the third an octave up. A result above `7F` diagnoses, as `.^` does.

- `.cC4mi01` answers `d4`, the minor third. (Orcvs writes a sharp as a lowercase letter, so `d4` is
  D♯4.)
- `.cC4Ma~.0104`: an up arpeggio, one tone per Tick, over four tones (C E G C).
- `.cC4M7~^0103`: up and down over four tones, using the proposed Bounce.
- `.cC4mi~?010005`: a random arpeggio over two octaves of a triad.
- Tones `01 02 03` make a first-inversion triad and `02 03 04` a second inversion, so voicings
  need no extra operand.

### `.s` Scale Degree: `.s root scale degree`

Answers the Note of degree `degree` (counted from `00`) of the scale on `root`. Degrees wrap by
octaves. Chords in a key come for free: tone `i` of the diatonic chord on degree `d` is degree
`d + 2i`. For example, `.sC4MA.+04.x~.010302` steps a triad on degree 4 (G B D in C major). That's
Sonic Pi's `chord_degree` without a separate Function.

Proposed two-Cell scale names: `MA` major, `mi` natural minor, `do` dorian, `ph` phrygian, `ly`
lydian, `mx` mixolydian, `lo` locrian, `hm` harmonic minor, `mm` melodic minor, `p5` major
pentatonic, `p3` minor pentatonic, `bl` blues, `wt` whole tone, `ch` chromatic.

### Quality names in two Cells

These names come from the orcvs-ts table, cut to what fits in two Cells. This is a draft for the
prototype to test, not a decision.

| Cells | Chord | orcvs-ts | Intervals |
|---|---|---|---|
| `Ma` | major | `M` | 0 4 7 |
| `mi` | minor | `m` | 0 3 7 |
| `dm` | diminished | `o` | 0 3 6 |
| `au` | augmented | — | 0 4 8 |
| `s2` `s4` | sus2, sus4 | — | 0 2 7, 0 5 7 |
| `M6` `m6` | major 6th, minor 6th | `M6` `m6` | 0 4 7 9, 0 3 7 9 |
| `M7` `m7` | major 7th, minor 7th | `M7` `m7` | 0 4 7 11, 0 3 7 10 |
| `D7` | dominant 7th | `7` | 0 4 7 10 |
| `mM` | minor-major 7th | `mM7` | 0 3 7 11 |
| `o7` `h7` | diminished 7th, half-diminished | `o7` `m7b5` | 0 3 6 9, 0 3 6 10 |
| `M9` `m9` `D9` | 9ths | `M9` `m9` `9` | add 14 |

There are two open points:

1. `D7` and `D9` look like Notes. The operand's type decides how the Cells read, as `C4` reads as a
   Number in an Add. But a reader could still take them for Notes.
2. Should the quality be a new operand type, or an Untyped operand that the Function checks?
   ADR 0024 makes the reading Function interpret the Cells.

### `!#` Chord Play (optional): `!# channel velocity root quality length`

Sends the Note On of every tone of the chord in one Turn, with Note Offs `length` Ticks later.
You can already build this from three `!~0064.cC4Ma0004`-style Plays with tones `00`, `01` and
`02`. Add `!#` only if writing three Plays turns out to be clumsy in practice.

### Strum

To spread a chord's Note Ons across a fraction of a Tick, Playback needs to schedule inside a Tick.
That's the same machinery as the timing doc's Ratchet `!r` and Swing, so strum waits for it. It
could become a `spread` operand on `!#`, or a Strum Function of its own.

## How chords use the timing proposals

| Musical idea | Orcvs, with the proposals |
|---|---|
| Arpeggio: up, up-down, random, ordered | `tone` from Clock `~.`, Bounce `~^`, Random `~?`, or Track `&t` |
| Arpeggio over octaves | a `tone` count past the chord size: `~.0106` on a triad spans two octaves |
| Progression, one chord per bar | `root` from `&t~.100404C4F4G4C4`, or step it on Every `~/` |
| Change chord every Nth hit, not every bar | Count `*+` drives the Track index (orcvs-ts's nested `ptn`) |
| Comping rhythm | a hex rhythm `~#` Banging the Plays, such as the Charleston `~#018200` |
| Thin out an arpeggio | Chance `?%` on the Bang feeding each Play (Tidal `degradeBy`) |
| Fill chords only in the last bar | During `~:` with And `?&` |
| Strum, ratchet an arpeggio | in-Tick scheduling (timing doc, section 5) |
| Velocity ramp across tones (orcvs-ts `seqL` options) | velocity from Interpolation `~>` or Bounce |

## Suggested order

1. **Scale Degree `.s`**: one Function that gives melodies in key, arpeggios and diatonic chords.
2. **Chord Tone `.c`** with the two-Cell quality table.
3. Bounce `~^` and Every `~/` from the timing doc, which make arpeggios and progressions short.
4. Chord Play `!#` and strum, only if they're needed, after in-Tick scheduling exists.

## Decisions for Toby

1. Should chord symbols go in a Payload (A), or should chords use the four-Cell form (B) or a
   bracketed literal (C)?
2. Should `!#` send a whole chord, which amends one output per Expression? Or should chords stay
   several Plays, each reading `.c`?
3. Do you want `.s` and `.c` as value Functions that answer one Note, or something else?
4. Two-Cell names for scales, or Number indexes into a table? Names read better;
   Numbers can be driven by a Clock to change scale.

## Sources

- orcvs-ts: `src/note.ts`, `src/sequence.ts`, `src/algo.ts`, `src/pulsar.ts`, `src/library.ts`,
  `src/globals.ts` (last commit 2023-04-20, "Pulse on every time or frame").
- orcvs-code: `snippets.json`, `README.md`.
- Orcvs: ADR 0024, ADR 0061, ADR 0063, ADR 0067, ADR 0074, ADR 0085, CONTEXT.md (Terminal Output Function), `docs/functions.md` (the Note operand type, `.^`).
- The descriptions of Tidal, Strudel, Sonic Pi, Ableton and Bitwig come from working knowledge of
  those tools, not from pages fetched for this note.
