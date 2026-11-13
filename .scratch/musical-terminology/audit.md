# Orcvs musical terminology audit

13 November 2026, against `main` at `97fb810`. Read-only: no repository changes. Each section below maps Orcvs Function, Role, type and spelling names to standard music-theory, MIDI and synth terms, with file:line citations. The headline mismatches were re-checked against the code.

## Is "chord type" the right term?

It is understood but informal. The standard term for what `M7`, `m7`, `7_`, `h7` and the rest name is **chord quality**; jazz pedagogy uses "quality" for the whole family, not only major, minor, diminished and augmented. The written suffix (`maj7`) is the **chord symbol** or **suffix**, and the semitone list is the chord's **formula**. The repo already avoids "chord type" (`CONTEXT.md:49`) and names the operand an **Interval Set**, because it also accepts scales. That umbrella is an Orcvs coinage but sound; it is not a pitch-class set, since it is ordered and runs past 12 semitones (14, 17, 21). Recommended doc wording: "an Interval Set: a chord quality or a scale, stored as its semitone formula above the root". Consider moving "chord quality" out of the avoid list and into the definition, since it is the word a musician searches for.

## Top findings

1. **`D5` and `D6` are Orcvs-only names.** They drop positions 2&3 and 2&4 from the top (`lang/src/functions/chord.rs:268-274`); jazz texts call these drop 2&3 and drop 2&4. The digit is the sum of the dropped positions, which is ambiguous (1+4 is also 5) and collides with the Notes D5 and D6. Lead the docs with the standard names; consider renaming the Rust variants `DropTwoAndThree` and `DropTwoAndFour`.
2. **Delay `~*` delays nothing.** It is a clock divider that fires when `tick % (length*steps) == 0` (`lang/src/functions/tick.rs:85-107`), named after Orca's `D`. The site's Sound Module shows an echo effect `DL` labelled "Delay" on the same page (`site/src/console/sound-module.ts:17`, `synth/src/delay.rs`). Rename one of them (for example `~*` Every, or the effect Echo).
3. **MIDI channels are 0-based and the docs never say so.** `00`–`0F` are MIDI channels 1–16, but `docs/functions.md:1038` gives only the range. The acid voice is on `09` (`synth/src/acid.rs:6`), which is MIDI channel 10, the General MIDI drum channel.
4. **The global effect CCs 1–8 collide with General MIDI** (`synth/src/lib.rs:224-231`): CC5 is Portamento Time, CC6 Data Entry, CC7 Channel Volume and CC8 Balance. The only `!c` example, `!c000740`, is half volume on external gear and a reverb level in the Built-in Synth.
5. **The drum map is not GM order, despite its comment** (`synth/src/drum.rs:13`). E plays Cowbell, G# Cymbal, A High Tom and B Conga, where GM has Electric Snare, Pedal Hi-Hat, Low Tom and Low-Mid Tom.
6. **Stack's `degree` counts from zero.** `docs/functions.md:390` says "degree `04` of C major is `G4`", which musicians call the 5th degree. `CONTEXT.md` also lists "Degree" as a term to avoid for Pick while describing Pick as picking a scale degree. Rename the Role to `index`, or give the 1-based degree alongside.
7. **Interpolation `~>` is a slew limiter.** It moves toward `target` by at most `amount` per Tick (`tick.rs:187-229`); "interpolation" means a value at a fraction between endpoints. "Slew" is missing from the glossary's avoid list. Rename to Slew, or say "slew limiter" in the docs.
8. **A Tick is a 16th note, but the reference never says so.** It is 4 per beat (`orcvs/src/opts.rs:12-17`); only `docs/first-result.md:80` states it, so a `length` of `04` cannot be read in beats from `docs/functions.md`.
9. **Monophonic Play `!%` always retriggers and so never slides** (`orcvs/src/playback/schedule.rs`); slide on the acid voice comes from overlapping Timed Plays. Musicians will expect the Monophonic Function to be the legato one. Separately, "slide", "glide" and portamento are mixed: `synth/src/glide.rs` is parameter smoothing, not portamento.
10. **"Length" means three things:** a Play's duration in Ticks (standard gate length), a period factor on Clock and Delay, and the Sound Module's `LN` decay scaler.

Smaller items: `p5` and `p3` for major and minor pentatonic have no stated mnemonic; Notes have sharps only (lowercase letters), so a flat-key musician reads E♭ as `d4`; the Euclidean pattern matches Bjorklund up to rotation but differs from published tables in about 42% of cases, and `docs/research/timing-operators.md:41` calls it Bjorklund; Pitch Bend docs lack the 14-bit meaning (centre 8192); Up/Down's `number` would read better as `semitones`; Reset All Controllers resets the acid sound controllers 70–79, which RP-015 says receivers should leave alone.

## Supported chords

| Code | Standard name | Symbol | Formula |
|---|---|---|---|
| `M_` | Major triad | C | 1 3 5 |
| `M7` | Major seventh | Cmaj7 | 1 3 5 7 |
| `M9` | Major ninth | Cmaj9 | 1 3 5 7 9 |
| `MD` | Major thirteenth | Cmaj13 | 1 3 5 7 9 13 |
| `M6` | Major sixth | C6 | 1 3 5 6 |
| `69` | Six-nine | C6/9 | 1 3 5 6 9 |
| `+M` | Augmented major seventh | Cmaj7♯5 | 1 3 ♯5 7 |
| `m_` | Minor triad | Cm | 1 ♭3 5 |
| `m7` | Minor seventh | Cm7 | 1 ♭3 5 ♭7 |
| `mM` | Minor-major seventh | Cm(maj7) | 1 ♭3 5 7 |
| `m6` | Minor sixth | Cm6 | 1 ♭3 5 6 |
| `m9` | Minor ninth | Cm9 | 1 ♭3 5 ♭7 9 |
| `mB` | Minor eleventh | Cm11 | 1 ♭3 5 ♭7 9 11 |
| `mD` | Minor thirteenth | Cm13 | 1 ♭3 5 ♭7 9 13 |
| `o_` | Diminished triad | C° | 1 ♭3 ♭5 |
| `o7` | Diminished seventh | C°7 | 1 ♭3 ♭5 𝄫7 |
| `h7` | Half-diminished | Cø7 | 1 ♭3 ♭5 ♭7 |
| `b9` | Dominant seventh flat nine | C7♭9 | 1 3 5 ♭7 ♭9 |
| `7_` | Dominant seventh | C7 | 1 3 5 ♭7 |
| `9_` | Dominant ninth | C9 | 1 3 5 ♭7 9 |
| `13` | Dominant thirteenth | C13 | 1 3 5 ♭7 9 13 |
| `s2` | Suspended second | Csus2 | 1 2 5 |
| `s4` | Suspended fourth | Csus4 | 1 4 5 |
| `+_` | Augmented triad | C+ | 1 3 ♯5 |
| `a9` | Added ninth | Cadd9 | 1 3 5 9 |

Scales: `MA` major, `mi` natural minor, `do` dorian, `ph` phrygian, `ly` lydian, `mx` mixolydian, `lo` locrian, `hm` harmonic minor, `mm` melodic minor (ascending), `p5` major pentatonic, `p3` minor pentatonic, `bl` blues, `wt` whole tone, `ch` chromatic.

---

# Part 1: Orcvs terminology audit: harmony and pitch

Read-only audit of `/home/user/orcvs`. Every claim cites file:line. "Standard" means common
lead-sheet / jazz-pedagogy / scientific-pitch usage.

## 1. Notes

| Fact | Evidence |
|---|---|
| Pitch letters `CcDdEFfGgAaB`: uppercase = natural, lowercase = sharp. No flats, no `e`/`b` (no E#/B#). | `lang/src/lib.rs:428`, `CONTEXT.md:162`, `docs/functions.md:22` |
| Octave cell: `/` = lowest octave (MIDI 0-11), then `0`-`9` = octave index 1-10. | `lang/src/lib.rs:432-446` |
| `C4` = `0x3C` = 60, so **middle C = C4**: scientific pitch notation (SPN) / MMA convention, not Yamaha (C3=60). `/` is SPN octave -1; top note `G9` = 127. | `lang/src/functions/chord.rs` test "C4M7 is ... 3C, 40, 43, 47"; `lang/src/functions/pitch.rs:214,222` |
| A Note is a MIDI note number 00-7F with a pitch-letter spelling. `.v`/`.^` convert to and from the plain Number. | `lang/src/atom.rs:7-16`, `:1303-1304`; `docs/functions.md:197-230` |
| Flats are always shown as sharps: the research doc itself calls `d4` (D#4) "the minor third" of C. | `docs/research/chords-and-patterns.md:203` |

Verdicts: SPN octave numbering = **match**. Sharp-only spelling = **acceptable** (MIDI-number
display, no key context), but a musician in a flat key reads `d4` for Eb4 and `a4` for Bb4.
Lowercase-for-sharp is **orcvs-specific**; a musician may guess lowercase means flat or means
nothing. "`/` for the octave below `0`" (`CONTEXT.md:162`, `docs/functions.md:22`) is
**acceptable**. Saying "octave -1" in the docs would help.

## 2. Interval Set (umbrella term)

`IntervalSet` is "A list of semitones above a root, named by a chord or a scale"
(`lang/src/atom.rs:254-259`, `CONTEXT.md:47-49`, ADR 0105). The list is ordered and rising
(`atom.rs:322-345`), it is indexed by position (Pick, Stack), and it holds compound intervals (14,
17, 21: `atom.rs:262,272-273`).

- Pitch-class set theory: a "set" is unordered pitch classes mod 12. An Interval Set is neither
  unordered nor mod 12, so set theory does not fit.
- Interval-class / interval vector: not this either.
- Standard terms: for a chord, **chord quality / chord type**, and its contents are the
  **chord formula** or interval structure; for a scale, **scale / mode**, and its contents are the
  **scale formula / step pattern**. No single standard word covers both. "Interval Set" is a
  reasonable Orcvs coinage for the union.
- `CONTEXT.md:49` says to avoid "Chord type, scale type, chord symbol". "Chord symbol" is rightly
  avoided, because a symbol includes the root. "Chord type" is the term a musician reaches for
  first.

**Verdict: orcvs-specific, acceptable.** Recommend doc wording: "an Interval Set is a chord
quality or a scale, stored as its formula in semitones above the root (rising, may pass the
octave)". Note that "set" here means a named list, not a pitch-class set.

## 3. Interval Set spellings and contents (`lang/src/atom.rs:260-298`)

All semitone contents are correct for the named chords and scales. Issues are in naming only.

| Orcvs | Rust name | Semitones | Standard symbol / name | Verdict | Recommendation |
|---|---|---|---|---|---|
| `M_` | Major | 0 4 7 | C, Cmaj, CΔ (triad) | acceptable | none |
| `M7` | MajorSeventh | 0 4 7 11 | Cmaj7, CM7, CΔ7 | match | none |
| `M9` | MajorNinth | +14 | Cmaj9 | match | none |
| `MD` | MajorThirteenth | 0 4 7 11 14 21 (no 11) | Cmaj13 (11 omitted as avoid note: standard) | orcvs-specific (hex D=13) | doc: "MD = maj13" |
| `M6` | MajorSixth | 0 4 7 9 | C6 ("major sixth" also names the interval) | acceptable | doc "6 chord" |
| `69` | SixNine | 0 4 7 9 14 | C6/9 | match | none |
| `+M` | AugmentedMajorSeventh | 0 4 8 11 | Cmaj7♯5, C+maj7 | match | none |
| `m_` | Minor | 0 3 7 | Cm | match | none |
| `m7` | MinorSeventh | 0 3 7 10 | Cm7 | match | none |
| `mM` | MinorMajorSeventh | 0 3 7 11 | Cm(maj7), CmM7 | acceptable; differs from `mm` (melodic minor) only by case, and a slip is not diagnosed (`CONTEXT.md:48`) | known hazard, already documented |
| `m6` | MinorSixth | 0 3 7 9 | Cm6 | match | none |
| `m9` | MinorNinth | 0 3 7 10 14 | Cm9 | match | none |
| `mB` | MinorEleventh | 0 3 7 10 14 17 | Cm11 | orcvs-specific (hex B=11) | doc already says so (`docs/functions.md:23`) |
| `mD` | MinorThirteenth | 0 3 7 10 14 21 | Cm13. **The 11th is often kept in m13** (it is not an avoid note over minor). | acceptable | doc: "11th omitted" |
| `o_` | Diminished | 0 3 6 | C°, Cdim | match | none |
| `o7` | DiminishedSeventh | 0 3 6 9 | C°7, Cdim7 | match | none |
| `h7` | HalfDiminished | 0 3 6 10 | Cø7, Cm7♭5 | match | none |
| `b9` | DominantFlatNinth | 0 4 7 10 13 | C7♭9 | acceptable: a bare "♭9" on a lead sheet can read as add♭9 | Rust/doc name: "dominant seventh flat nine (7♭9)" |
| `7_` | DominantSeventh | 0 4 7 10 | C7 | match | none |
| `9_` | DominantNinth | 0 4 7 10 14 | C9 | match | none |
| `13` | DominantThirteenth | 0 4 7 10 14 21 (no 11) | C13 (11 omitted: standard) | match | none |
| `s2`/`s4` | SuspendedSecond/Fourth | 0 2 7 / 0 5 7 | Csus2/Csus4 | match | none |
| `+_` | Augmented | 0 4 8 | C+, Caug | match | none |
| `a9` | AddNine | 0 4 7 14 | Cadd9 | match | none |
| `MA` | MajorScale | Ionian | major / Ionian | match | none |
| `mi` | NaturalMinorScale | Aeolian | natural minor / Aeolian | match | none |
| `do ph ly mx lo` | modes | correct | Dorian, Phrygian, Lydian, Mixolydian, Locrian | match | none |
| `hm` | HarmonicMinorScale | 0 2 3 5 7 8 11 | harmonic minor | match | none |
| `mm` | MelodicMinorScale | 0 2 3 5 7 9 11 | melodic minor, **ascending form** (jazz minor); classical descending = Aeolian | acceptable | doc: "ascending / jazz melodic minor" |
| `p5` | MajorPentatonicScale | 0 2 4 7 9 | major pentatonic | **mismatch (unexplained mnemonic)** | rename (e.g. `pM`/`pm`, though case-pairing repeats the `mM`/`mm` hazard) or document the mnemonic |
| `p3` | MinorPentatonicScale | 0 3 5 7 10 | minor pentatonic | **mismatch**: "3" suggests a third or three tones; no rationale anywhere (`docs/research/chords-and-patterns.md:219-220`, `.scratch/chords-and-pitch/spec.md:140`) | same as above |
| `bl` | BluesScale | 0 3 5 6 7 10 | (minor) blues scale; a "major blues" 0 2 3 4 7 9 also exists | acceptable | doc: "minor blues hexatonic" |
| `wt`, `ch` | WholeTone, Chromatic | correct | match | match | none |

The naming rules in ADR 0105 (`docs/adr/0105-...md`, "Names follow three rules") are drop the implied 7, use an established symbol, and write hex B/D for 11/13. The hex rule
is invented and must be learned, but it is documented (`docs/functions.md:23`).

Gaps a musician would notice: no 7sus4, 7♯9, 7♯11/maj7♯11, dominant 11, or m7♭5 under that symbol (it
exists as `h7`). Not errors.

## 4. Voicings (`lang/src/atom.rs:350-360`, algorithm `lang/src/functions/chord.rs:161-301`)

Fixed order of operations: the Voicing removes tones (`chord.rs:182-187`), inversion raises the lowest
tone an octave n times (`chord.rs:191, 220-227`), OP raises every 2nd kept tone (`chord.rs:193-197`), the
drops lower positions counted from the top starting at 1 (`chord.rs:201-204`), and then the tones are sorted.

| Orcvs | What it does (code) | Standard term | Verdict | Recommendation |
|---|---|---|---|---|
| `CL` Close | nothing (`chord.rs:245-250`) | close position | match | none |
| `OP` Open | raises positions 1, 3, 5... from the bottom an octave (`chord.rs:194`, `:263`). Triad C E G becomes C G E' (spread triad, standard). 7th C E G B becomes C4 G4 E5 B5, **the same shape as `D6` an octave higher** (D6 gives C3 G3 E4 B4). | "open position" is a loose term (classical: gaps in the upper voices); spread voicing | acceptable, orcvs-specific procedure. **Redundant with D6 for 4-tone chords.** | doc: "spread: every other tone up an octave; for 4-tone chords equals D6 an octave up" |
| `D2` | lowers 2nd from top (`chord.rs:271`) | drop 2 | match | none |
| `D3` | lowers 3rd from top (`chord.rs:272`). On a triad it drops the root (C3 E4 G4); drop 3 is normally defined for 4 voices. | drop 3 | match (4 voices); nonstandard on triads | doc note |
| `D5` DropFive | lowers 2nd **and** 3rd from top (`chord.rs:273`) | **drop 2+3** ("drop 2 and 3") | **mismatch / invented**. "Drop 5" is not a jazz term; the digit is the *sum* of positions (`chord.rs:268`, `docs/functions.md:1169`, ADR 0103). The sum is ambiguous (1+4 = 5 as well) and the spelling collides with Note `D5` / Number `D5` (`CONTEXT.md:52`). | rename `D23` cannot fit in 2 Cells. Options: Rust variant `DropTwoAndThree`, docs lead with "drop 2&3", or pick a spelling such as `Dt`/`DT`. At minimum change the Rust name and the glossary wording. |
| `D6` DropSix | lowers 2nd and 4th from top (`chord.rs:274`) | **drop 2+4** ("drop 2 and 4") | **mismatch / invented**, same as D5 | Rust variant `DropTwoAndFour`, docs "drop 2&4" |
| `SH` Shell | keeps positions 0, 1, 3 (`chord.rs:243`): R-3-7 on 7th/9th/13th chords, R-3-6 on 6 chords and 6/9, R-♭3-𝄫7 on o7, degrees 1-2-4 on a scale. Needs 4 tones, so triads are refused. | shell voicing (Bud Powell): R-3-7 **or R-7-3** | acceptable. The common left-hand R-7-3 form (R-7-10) **cannot be reached**, because SH and OP are one operand and inversion only rotates (3-7-R, 7-R-3). | doc note; possible future voicing |
| `RL` Rootless | removes position 0 only (`chord.rs:244`) and keeps the 5th: triad gives a 3-5 dyad, 7th gives 3-5-7, 9th gives **3-5-7-9 = Bill Evans "A" form**; with inversion 2, 7-9-3-5 = **"B" form** | rootless voicing (A/B forms, 3-7-9-(5/13)) | acceptable; matches A/B on 9th chords only | doc: "on a 9th chord RL gives the A form, and inversion 02 gives the B form" |

Drops on 5- and 6-tone chords count from the top of *all* tones, extensions included
(`chord.rs:201-204`). Drop voicings are defined for 4 voices in jazz texts, so this is a logical
extension but nonstandard. Inversion before drop (`chord.rs:153-160`) matches the standard
derivation: the drop-2 family is the drop-2 of each close inversion.

**Inversion** (`CONTEXT.md:59-60`, `chord.rs:216-227`): lowest tone up an octave, n times = **standard**.
Values 00-FF are accepted (`docs/functions.md:1045`). An inversion count at or above the tone count is an octave
transposition, not an inversion in the theory sense: **acceptable**. Doc: "inversion n ≥ tone count
also moves the chord up octaves". Inversion counts tones *after* SH/RL removal, so "first inversion" of
an SH chord puts the 3rd in the bass: consistent.

## 5. Pitch Functions (`lang/src/atom.rs:1303-1347`, `lang/src/functions/pitch.rs`)

| Orcvs | What it does | Standard term | Verdict | Recommendation |
|---|---|---|---|---|
| `#+` Up, `#-` Down (`atom.rs:1315,1347`; `pitch.rs:40-54`) | note ± n semitones, out of range diagnoses | transpose up/down | match | rename role `number` to `semitones` (descriptions already say "number semitones") |
| `#p` Pick `root set index` (`atom.rs:1327`; `pitch.rs:68-93`) | zero-based index into the set, wraps up one octave per pass (`pitch.rs:70-71`). Index 00 = root (`docs/functions.md:315`). Not monotonic for sets wider than an octave (`#pC4MD05`=A5, `06`=C5; `CONTEXT.md:182`). | "nth scale degree" / "nth chord tone" (1-based in theory: root = 1st degree / "1") | acceptable (programmer indexing) | doc should always pair the index with its degree, e.g. "index 04 = 5th degree (G)" |
| `#s` Stack `set degree size` (`atom.rs:1340`; `pitch.rs:102-136`) | takes every other entry starting at `degree` (`pitch.rs:133`): **tertian stacking on 7-note scales** (diatonic 7ths verified, `pitch.rs:301-308`). Answers an **Interval Set (quality), not notes**. On pentatonics/chords "every other entry" is not thirds (C maj pent gives 0 4 9, `docs/functions.md:377-378`). Fails where no name matches: a major-scale degree-0 size 6 (maj11) or any 7-tone 13th (the table's 13ths omit the 11th), so `13`/`MD`/`mD` are never answered. | diatonic chord on scale degree / harmonizing the scale / chord quality of degree N (Roman-numeral analysis) | acceptable function, **mismatched role name** | see below |
| `degree` role of Stack | **zero-based**: `docs/functions.md:390-392` says "degree `04` of C major is `G4`". Musicians call G the **5th degree (V)**. | scale degree (1-based, I-VII) | **mismatch** | rename the role to `index` (as Pick) or say "index 04 (degree V)". Also `CONTEXT.md:183` lists "Degree" as an *avoided* term for Pick, while Stack's role is `degree` and the Pick entry itself says it "picks a scale degree" (`CONTEXT.md:182`). The glossary contradicts itself. |
| Stack name | collides with "Operand Stack" (`CONTEXT.md:95-97`, `:186` "Not the Operand Stack") | "harmonize", "diatonic chord", "chord on degree" | orcvs-specific | acceptable (tertian "stacking" is real vocabulary); keep the disambiguation |
| `.^`/`.v` Convert to Note/Number | MIDI number <-> Note | MIDI note number | match | none |
| `!:` Raw / `!#` Timed Chord Play, roles `root set voicing inversion` (`atom.rs:1331,1343`) | sounds root + set, voiced | chord = root + quality; "voicing", "inversion" | match | none |

## 6. Summary of recommended changes (doc or rename)

1. D5/D6: invented "sum of positions" names. Standard is **drop 2&3** and **drop 2&4**. At least
   rename the Rust variants (`DropTwoAndThree`, `DropTwoAndFour`) and lead the docs and glossary with
   the standard names. Consider a spelling that does not also read as Note D5/D6.
2. Stack's `degree` is zero-based and docs call G "degree 04" of C. Either make the wording "index"
   or always gloss it with the Roman/1-based degree. Fix the CONTEXT.md contradiction (Pick avoids
   "Degree" yet "picks a scale degree"; Stack uses `degree`).
3. `p5`/`p3`: unexplained, `p3` is misleading. Rename or document.
4. OP = D6 shape (+1 octave) on 4-tone chords: document, or redefine OP.
5. SH cannot produce R-7-3; RL keeps the 5th (A form only on 9ths). Document.
6. `mm` = ascending/jazz melodic minor; `bl` = minor blues; `mD` omits the 11th. Document.
7. Note spelling: sharps only, lowercase = sharp; say "C/ = C-1 (MIDI 0); C4 = middle C = 60".
8. Interval Set: keep, but describe it as "chord quality or scale, as a semitone formula above the
   root" so musicians do not read it as a pitch-class set.
9. Up/Down role `number` would read better as `semitones`.

---

# Part 2: Timing, rhythm and sequencing

Read-only audit of /home/user/orcvs. All paths relative to the repo root.

## Ground truth: what a Tick is

- `orcvs/src/opts.rs:12-13` `TICKS_PER_BEAT = 4`; `opts.rs:84-85` one Tick = 125 ms at 120 BPM. So a
  Tick is a **16th note** at the displayed BPM (fixed 4 PPQN), exactly like Orca's frame
  (`docs/research/timing-operators.md:31-33`, `:71-72`).
- `CONTEXT.md:241-243` defines Tick as "One discrete musical-time step", avoids "Cycle, frame" — but
  never states that it is a 16th note / a quarter of a beat.
- `CONTEXT.md:437-439` BPM is "The performer's display of the Tick Grid's period" — doesn't state
  the 4:1 ratio either.
- The only user-facing statement of the ratio is `docs/first-result.md:80` ("a beat is four Ticks").
  `docs/functions.md` (the reference) uses "Tick" ~everywhere (`:40`, `:51-52`, `:409-411`) and never
  says how long one is.
- Collision: in DAW/MIDI vocabulary a "tick" is a PPQN sub-division (typically 1/96–1/960 of a
  quarter). In Orcvs it is the coarse sequencer step (a 16th). A musician reading "length `04`
  Ticks" cannot tell whether that is a 16th or a quarter note without first-result.md.

## Table

| Orcvs name | What it does (file:line) | Standard term(s) | Verdict | Recommendation |
|---|---|---|---|---|
| **Tick** | one 16th-note step; 4 per BPM beat (`orcvs/src/opts.rs:12-17`; `CONTEXT.md:241`) | step, 16th, pulse (sequencer); "tick" in DAWs = PPQN sub-division | acceptable (Orca lineage) but under-documented | Add "a Tick is a 16th note: four Ticks make a beat at the displayed BPM" to `CONTEXT.md:242`, `CONTEXT.md:438` and the "How to read an entry" section of `docs/functions.md:11-52`. Optionally express examples in beats/bars ("`10` Ticks = one 4/4 bar"). |
| **BPM** | displayed tempo; Tick period = 60 s / (4·BPM) (`opts.rs:84-96`) | BPM, tempo | match | State the quarter-note = 4 Ticks relationship in the CONTEXT entry. |
| **Clock** `~.` [length, steps] | `floor(tick/length) % steps`, a Number (`lang/src/functions/tick.rs:60-83`; `atom.rs:1300`; `docs/functions.md:414-418`) | step counter, divided counter, playhead/step index; Orca `C` "clock" (`timing-operators.md:38,71`) | orcvs-specific / mismatch for modular users | In modular/DAW usage a "clock" is a pulse train, not a number; this is a clock divider feeding a counter (a sequencer's step position). Keep the name for Orca parity but document it as "a step counter driven by a divided clock: answers the current step index". CONTEXT `_Avoid_: Counter` (`CONTEXT.md:195`) avoids exactly the word a musician would search for. |
| Clock `length` role | Ticks per step (`tick.rs:62-64`) | rate, division, step length, clock divide | acceptable, but collides | Same word as Play `length` (note duration) and the Sound Module's `LN` Length effect. Consider `rate` or `division`, which is what `docs/research/timing-operators.md:38-39` already calls it. |
| Clock `steps` role | steps per cycle (wrap) (`tick.rs:62-64`) | steps, pattern length, modulus | match | — |
| **Delay** `~*` [length, steps] | Bang when `tick % (length*steps) == 0`, from Tick 0 (`tick.rs:85-107`; `atom.rs:1309`; `docs/functions.md:436-439`) | clock divider, periodic trigger, metro (Max), "every n" (Tidal); Orca `D` "delay" (`timing-operators.md:39,71`) | **mismatch** | Nothing is delayed: it fires on Tick 0 and is a pure divider with no phase (`timing-operators.md:52-53`). To a musician "delay" is an echo effect or a note offset/nudge. Rename (e.g. Divide/Every/Pulse/Metro — the research doc's proposed `~/` "Every" at `timing-operators.md:171-189` already calls it "the clock-divider idea" and says Delay would become a special case) or at least describe it in `docs/functions.md:438` as "a clock divider: one Bang every `length*steps` Ticks". |
| Delay `length`, `steps` roles | factors whose product is the period (`tick.rs:87-92`) | division / period / rate | mismatch | Two factors for one period is Orca's `D rate mod`; "steps" here doesn't count steps of anything that sounds. If kept, explain "period = length × steps Ticks; `steps 01` gives one Bang per `length` Ticks". |
| Delay vs synth `DL` Delay (collision) | site Sound Module shows effect `DL` with hover word "Delay" (`site/src/console/sound-module.ts:17`), an echo on CC 4 (`synth/src/delay.rs:1`, `synth/src/lib.rs:227`; `docs/adr/0088…:38`) | delay = echo | **user-facing collision** | Same page (`/`) shows both the `~*` "Delay" Function and the "Delay" echo effect. CONTEXT already avoids "delay line, echo" for `~*` (`CONTEXT.md:199`) but has no rule for the effect. Strong reason to rename `~*`, or rename the effect "Echo". Also `CONTEXT.md:370` uses "a fixed delay" for audio latency — a third meaning. |
| **Euclidean** `~%` [hits, steps] | Orca bucket formula: Bang when `(hits*phase)%steps + hits >= steps`, `phase=(t%steps+steps-1)%steps` (`tick.rs:109-152`; ADR `docs/adr/0012…:7`) | Euclidean rhythm E(k,n) (Toussaint), Bjorklund; k = pulses/onsets/hits/fills, n = steps | match (names) / nuance (rotation) | `hits`/`steps` are standard Eurorack wording. Pattern always starts on an onset; `~%0308` = `X..X..X.` (`docs/functions.md:458-460`). Verified (scratch script, n≤32): the bucket output is always the same necklace as Bjorklund but a **different rotation in ~42% of (k,n)** (237 of 560), e.g. E(2,5) gives `x..x.` vs Toussaint's `x.x..`, E(5,8) `x.x.xx.x` vs `x.xx.xx.`. `timing-operators.md:41` calls it "Orca's Bjorklund distribution" — inaccurate; ADR 0012 correctly says "bucket distribution". No rotation operand (`timing-operators.md:63-64`, proposal `:237-246`). Document "evenly distributed, starting on an onset; rotations may differ from published Bjorklund tables". |
| **Increment** `~+` [amount, modulus] | `(previous + amount) % modulus`, feedback via its own Output Portal, advances every Tick (`tick.rs:154-185`; `docs/functions.md:482-486`) | counter, accumulator, step counter with wrap; Orca `I` | acceptable | Every musician/modular user will call this a counter (wrapping accumulator). CONTEXT avoids both "Counter, accumulator" (`CONTEXT.md:207`), so the docs never use the searchable word. Add "a wrapping counter" to the doc sentence. Note it advances per Tick (whenever evaluated), not per trigger, unlike a modular counter clocked by a gate. `modulus` is math-speak; "wrap"/"length" is musician wording — acceptable. |
| **Interpolation** `~>` [amount, target] | moves previous toward `target` by ≤ `amount` per Tick, no overshoot — a linear, rate-limited ramp (`tick.rs:187-229`; `docs/functions.md:504-507`) | slew limiter (linear), ramp, glide/portamento (pitch), lag (usually exponential); Orca `Z` "lerp" (`timing-operators.md:43,71`) | **mismatch** | "Interpolation" means computing a value between two endpoints from a fraction (lerp); this has no fraction and no fixed endpoints, it is a constant-rate slew. CONTEXT avoids "Lerp, tween, easing, filter, glide" (`CONTEXT.md:211`) but none of those is the right word either; "slew" is the standard one. Rename to Slew/Ramp, or describe as "a slew limiter: moves toward target at most `amount` per Tick". Collision: the synth has a `Glide` type doing the same thing in audio time (`synth/src/glide.rs:1-9`) — internal only. |
| Interpolation `amount` role | max step per Tick | rate, slew rate, speed | acceptable | `rate` (as the research doc says, `timing-operators.md:43`) is clearer than `amount`. |
| **Random** `~?` [seed, minimum, maximum] | deterministic number from seed, absolute Tick and Position; a new value every Tick (`tick.rs:231-300`; ADR 0013; `docs/functions.md:526-530`) | random, (stepped) random voltage, sample & hold of noise at Tick rate, seeded RNG | acceptable | A new value each Tick = S&H clocked at the Tick (16th). Docs are clear that it is reproducible; consider adding "a new value every Tick (like sample-and-hold clocked at the Tick rate)". CONTEXT avoids "noise, dice" (`CONTEXT.md:215`), fine. `seed`/`minimum`/`maximum` — match. |
| **Track** `&t` [index, count] | reads pair `index % count` from the pairs after it (`lang/src/functions/track.rs:7-16`; `docs/functions.md:729-732`; `CONTEXT.md:143-144`) | step sequencer read / sequence lookup, "select", array index; Orca `T` "track" | orcvs-specific | In DAWs a "track" is a lane/channel, not a lookup. With Clock driving `index`, `&t` is the classic step-sequencer readout. Document as "reads step `index` of a `count`-step sequence written after it". |
| **Push** `@t` [index, count, value] | writes `value` into pair `index % count` of the row below (`atom.rs:1325`; `docs/functions.md:826-829`; `CONTEXT.md:154-155`) | step record / write into sequence, "push" (Orca `P`) | orcvs-specific | "Push" suggests a stack/queue push; it is an indexed write (step record). Orca lineage; doc wording acceptable. |
| **Timed Play** / **Monophonic Play** / Timed Chord Play `length` | Note Off `length` Ticks after Note On (`atom.rs:1338,1345,1316`; `CONTEXT.md:329-330`; `docs/functions.md:1099-1102`, table `:1042`) | gate length, note length/duration | match | Standard. But unit is Ticks (= 16ths) and is never converted for the reader; `04` = one beat. Type `Length` (`lang/src/atom.rs:158-166`) matches. Collides in name with the Sound Module `LN` "Length" effect (decay length scale, CC 3; `site/src/console/sound-module.ts:16`; `docs/adr/0088…:37`) and with Clock/Delay's `length` role (a period, not a duration). "Monophonic Play" — match (mono voice, last-note steals). |
| **Bang** `**` | transient pulse that activates aligned roots in the same Tick (`CONTEXT.md:115-117`; `docs/functions.md:846-850`) | trigger (modular), trig (Elektron), bang (Max/Pd) | match (Max/Pd/Orca) | CONTEXT avoids "trigger flag" (`CONTEXT.md:117`) — fine; a gloss "a trigger" in functions.md would help non-Max users. Not a gate (no duration). |
| **Halt** `*!` | stops the root directly south from taking its Turn that Tick (`atom.rs:1312`; `docs/functions.md:1010-1013`; `CONTEXT.md:135-137`) | mute, inhibit, choke, skip | acceptable (Orca `H`) | "Halt" reads like stopping the transport. Doc says "stops the root ... that Tick", which is clear; consider "mutes" in the doc sentence. |
| **Missed Bang** | Bang reaching a root that already took its Turn (`CONTEXT.md:123-125`) | — | orcvs-specific | fine (an implementation/diagnostic term). |
| **Run Clock** | wall-clock time since Tick 0 (`CONTEXT.md:441-443`) | transport position / elapsed time | acceptable | A second "Clock" with different meaning from `~.` Clock; no conflict in practice. |

## Doc drift (not naming, but confusing)

- `docs/research/timing-operators.md:38-43` labels operands with names the code does not use: Clock/Delay
  `rate modulus` (code: `length steps`, `atom.rs:1300,1309`), Increment `step modulus` (code:
  `amount modulus`), Interpolation `rate target` (code: `amount target`). Either the research doc is
  stale or it shows the musician-friendlier names; worth reconciling.
- `docs/research/timing-operators.md:41` says Euclidean uses "Orca's Bjorklund distribution"; ADR 0012
  and `tick.rs:111` correctly say bucket distribution (not Bjorklund's exact rotations).
- `docs/research/timing-operators.md:44` describes Halt as "stops the root south of it for one Tick" —
  consistent with functions.md.

## Answers to the brief's questions

- **Tick vs step/PPQN:** a Tick is a 16th-note step (4 PPQN), not a DAW tick. Clear only in
  `docs/first-result.md:80`; missing from CONTEXT.md and functions.md.
- **length in Ticks:** standard gate length / note duration for Plays; but Clock/Delay reuse `length`
  for a period, and the Sound Module reuses "Length" for decay scaling.
- **Euclidean:** E(k,n) with standard `hits/steps` naming; always a rotation of the Bjorklund/Toussaint
  necklace, starting on an onset, differing from the published rotation in ~42% of cases; no
  rotation operand.
- **Clock:** matches Orca `C` (`floor(t/rate) % mod`); musically a clock-divided step counter, not a
  "clock".
- **Delay:** matches Orca `D` but is a clock divider; misleading name, and collides on the same site
  page with the synth's "Delay" (echo) effect `DL`.
- **Interpolation:** a linear slew limiter / ramp; "interpolation" (and Orca's "lerp") is the wrong term.
- **Increment:** a wrapping counter/accumulator, advanced every Tick.
- **Random:** seeded, per-Tick stepped random (S&H at Tick rate); name fine.

---

# Part 3: MIDI, Built-in Synth, effects, OSC and UDP

Read-only audit of /home/user/orcvs. Standard references: MIDI 1.0 Detailed Spec, GM1 / GM2 CC and drum map, RP-015 (Reset All Controllers), OSC 1.0, Roland TB-303 / TR-808 panel names. Where a claim relies on a standard from memory, it says so.

## Top findings (ranked)

1. **Channel numbering is 0-based hex, and the docs never say so.** `MidiChannel` is `00`–`0F` (lang/src/atom.rs:37, :62); docs/functions.md:1038 and CONTEXT.md:326/:370 state ranges only in this form. MIDI 1.0 is wire-0-based but every DAW, synth panel and GM document shows channels **1–16**. Nowhere does the user-facing text say "`00` is MIDI channel 1". Consequence: the acid channel "`09`" is **MIDI channel 10**, the GM percussion channel.
2. **The 303 voice sits on GM's drum channel.** acid.rs:6 `CHANNEL = 9`; synth/src/lib.rs:60-61; ADR 0099:17-20; CONTEXT.md:370. The kit is on `08` and `0A`–`0F` (= MIDI 9, 11–16). A musician who sends a drum pattern on "channel 10" from a DAW gets basslines. The rationale (ADR 0099:52-56) is about not losing a tone slot; the GM collision is never mentioned.
3. **Global effects use CC 1–8, which collide with GM assignments.** synth/src/lib.rs:224-231, ADR 0088 table. CC2 Breath, CC4 Foot, **CC5 Portamento Time** -> chorus, **CC6 Data Entry MSB** (the RPN data byte, e.g. pitch-bend range) -> tilt EQ, **CC7 Channel Volume** -> small reverb, **CC8 Balance** -> large reverb. GM's own sends are CC91 reverb and CC93 chorus. The source is Orca/Learn Orca's Enfer (ADR 0088:8-9), so this is Orcvs/Enfer convention, not MIDI. These effects are also *global* (they ignore the channel), while every GM CC is per-channel. The docs' only `!c` example, `!c000740` (docs/functions.md:1207-1214), is CC7 = 0x40: half volume on external gear and a reverb level on the Built-in Synth.
4. **Drum map says "in General MIDI's order from C", but only 8 of 12 pitch classes match GM.** drum.rs:13-47 maps by `note % 12` (C4 = 60, lang/src/atom.rs:3002, so GM key 36 = C2 = pitch class 0). Matches: 36 Kick, 37 Rim (GM Side Stick), 38 Snare, 39 Clap, 41 Low Tom (GM Low Floor Tom), 42 Closed Hat, 43 Mid Tom (GM High Floor Tom, close), 46 Open Hat. Mismatches: 40 E -> **Cowbell** (GM Electric Snare; GM Cowbell is 56), 44 G# -> **Cymbal** (GM Pedal Hi-Hat), 45 A -> High Tom (GM Low Tom), 47 B -> **Conga** (GM Low-Mid Tom). Because the octave is ignored, GM keys outside 36–47 land on unrelated voices: 49 Crash -> Rim, 51 Ride -> Clap, 56 Cowbell -> Cymbal, 64 Low Conga -> Cowbell.
5. **Reset All Controllers resets the acid sound controllers.** acid.rs:44-55 (CC121 -> defaults); ADR 0099:47-49. RP-015 (from memory) lists Sound Controllers CC70–79 among those a receiver should *not* reset. It is an intentional choice, but it is non-standard, and the safety action (orcvs/src/midi.rs:274-279) sends CC121 on every stop.
6. **Monophonic Play is not "mono/legato" in the synth sense.** It is sender-side voice management: retrigger mono with last-note priority, and it always sends Note Off before Note On (orcvs/src/playback/schedule.rs:27, :198; CONTEXT.md:342; docs/functions.md:1117-1121). So on the acid channel it **never slides** (ADR 0099:51-52), and the slide comes from overlapping *Timed* Play. A musician will expect the "Monophonic" Function to be the legato/slide one. MIDI "Mono Mode" (CC126, Mode 4) is a receiver mode and unrelated.
7. **Slide / glide / portamento are used loosely.** The 303 term is *slide*: acid.rs:13 `SLIDE`, ADR 0099:11, CONTEXT.md:370. Yet ADR 0099:22 says "the pitch glides" and the test is named `..._glides_to_its_pitch` (synth/src/lib.rs:908). Separately, `glide.rs` / `struct Glide` (glide.rs:1-9) and `vibrato.rs:14 GLIDE` are **parameter smoothing (slew)**, not portamento. GM's portamento is CC5/CC65/CC84, and CC5 here is chorus.
8. **"Length" means three things.** It is a Play's lifetime in Ticks (`Length`, atom.rs:159-166), the `LN` effect that scales decay/release (lib.rs:52, ADR 0088 table), and the per-voice `length` scaling factor (drum.rs:50-52, tone.rs:67, acid.rs:72). Standard words: gate time / duration for the first, decay / release for the second.
9. **Pitch Bend docs give no 14-bit meaning.** docs/functions.md:1223 says only "low byte first. `!b000040` is the centre". It never gives value = msb*128 + lsb, the range 0–16383, centre 8192, or that the bend range is the receiver's (RPN 0). CONTEXT.md:338 is accurate but wire-centric. The Built-in Synth also ignores Pitch Bend (lib.rs:65-67), and CONTEXT.md:370 doesn't say so.

## Table

| Orcvs name | What it does (file:line) | Standard term(s) | Verdict | Recommendation |
|---|---|---|---|---|
| Raw Play `!>` | Note On; vel `00` -> Note Off 0x80 (atom.rs:1332; midi.rs:309-320) | Note On / Note Off | orcvs-specific | Keep; gloss as "Raw Play (Note On, you send the Note Off)" in docs/functions.md:1070. "Raw" means "no lifetime", which isn't a MIDI word. |
| velocity `00` = stop | Engine converts to 0x80 Note Off, never Note On vel 0 (CONTEXT.md:318; midi.rs:309-315) | Note On vel 0 is the spec's Note Off equivalent (often used with running status) | match | None. The synth also accepts 0x9n vel 0 as release (lib.rs:222-223). |
| Timed Play `!~` | Note On + Note Off after N Ticks (atom.rs:1344) | Note with duration / gate length | acceptable | Doc could say "gate length in Ticks". |
| Monophonic Play `!%` | Per-channel retrigger mono, Note Off before Note On (atom.rs:1324; schedule.rs:27) | mono (retrigger, last-note priority); not legato | mismatch (expectation) | Say "retriggers, never legato/slides" in docs/functions.md:1117; point to Timed-Play overlap for 303 slide. |
| "Monophonic note" / "Mono voice" | The note a channel's `!%` owns (CONTEXT.md:342) | mono voice | acceptable | Distinguish from synth `VOICES` polyphony (lib.rs:30). |
| Raw/Timed Chord Play `!:` `!#` | One Note On per tone (atom.rs:1331, :1343) | chord | match | None. |
| Voicing `D5` `D6` | drop 2+3 / drop 2+4 (docs/functions.md:1166-1172) | "Drop 2&3", "Drop 2&4" | orcvs-specific | Already explained; keep the explanation beside the names. |
| `channel` / `MidiChannel` `00`–`0F` | 0-based (atom.rs:37, :62) | MIDI channel 1–16 as displayed | mismatch (doc) | Add "`00`–`0F` is MIDI channels 1–16" to docs/functions.md:1038 and CONTEXT.md:326/:370. |
| `velocity` / `Velocity` | 00–7F (atom.rs:131) | Velocity | match | — |
| Control Change `!c`, `controller`/`Controller` | CC number (atom.rs:1302, :133) | Control Change, controller number ("CC#") | match | Optionally "controller (CC number)". |
| `value` / `ControlValue` | CC data (atom.rs:143) | control value | match | — |
| Pitch Bend `!b`, `lsb`/`msb` (`BendLsb`/`BendMsb`) | Raw wire bytes (atom.rs:1328, :145-155) | Pitch Bend Change, 14-bit value, centre 8192 (0x2000) | acceptable | Add the msb*128+lsb, 0–16383, centre 8192 sentence and a down/up example to docs/functions.md:1223. |
| `length` / `Length` | Ticks 00–FF (atom.rs:159-185) | duration / gate time | acceptable | Avoid "length" for the LN effect (see LN). |
| Safety action | CC123, CC121, bend 0x2000 per channel (midi.rs:274-279) | panic / All Notes Off / Reset All Controllers | match | — |
| CC120 All Sound Off | Treated as a release (lib.rs:233) | All Sound Off = immediate mute | mismatch (minor) | Cut voices instantly, or document it as release. |
| Built-in Synth tone channels 0–7 | Sine, Triangle, Saw, Square, FM, FM, AM, AM (tone.rs:44-63) | waveform names, 2-op FM, AM | match | Internal `Shape` vs acid `waveform`: pick one word. |
| Tone envelope | 5 ms attack, 150 ms release (tone.rs:5-10) | AR envelope | match | — |
| Acid voice (`Acid`, "acid channel") | 303-style mono voice, ch `09` (acid.rs:1-6) | TB-303 bass line | acceptable | Rename nothing; document "MIDI channel 10" and the GM-drum clash. |
| acid `waveform` CC70 | saw < 40, square >= 40 (acid.rs:48-50) | 303 "Waveform" switch; GM2 CC70 Sound Variation | match | — |
| acid `resonance` CC71 | (acid.rs:51) | 303 Resonance; GM2 CC71 Timbre/Harmonic Intensity (resonance) | match | — |
| acid `cutoff` CC74 | (acid.rs:52) | 303 "Cutoff Freq"; GM2 CC74 Brightness | match | — |
| acid `decay` CC75 | Filter envelope decay (acid.rs:53, :122-131) | 303 Decay; GM2 CC75 Decay Time | match | — |
| acid `env_mod` CC79 | Envelope depth (acid.rs:54) | 303 "Env Mod"; GM2 CC79 undefined sound controller | match | — |
| Accent (velocity >= 0x78) | Louder, brighter, shortest decay (acid.rs:17, :79, :124-126) | 303 Accent (per step) + Accent knob | acceptable | No accent-amount control and no Tuning; document as "fixed accent". |
| slide (`SLIDE`) | Legato overlap glides pitch ~60 ms (acid.rs:13-14, :94-103) | 303 Slide | match | Use "slide" consistently: ADR 0099:22 "glides", lib.rs:908 test name. |
| `Glide` (glide.rs), `GLIDE` (vibrato.rs:14) | Parameter smoothing to avoid clicks (glide.rs:1-9) | slew / smoothing / ramp | mismatch | Rename to `Smoothed`/`Slew`, since "glide" means portamento to musicians. |
| Kit `Instrument` | 12 voices by pitch class (drum.rs:13-47) | TR-808 / GM drum map | mismatch (partial) | Fix the "General MIDI's order" comment (drum.rs:13) or align E/G#/A/B; consider GM-key mapping. |
| `Kick` | (drum.rs:17) | 808 "Bass Drum"; GM "Bass Drum 1" | acceptable | — |
| `Rim` | (drum.rs:18) | 808 "Rim Shot"; GM "Side Stick" | acceptable | — |
| `Snare` | | 808 "Snare Drum" | match | — |
| `Clap` | | 808 "Hand Clap" | match | — |
| `Cowbell` on E | (drum.rs:21, :37) | 808 "Cow Bell"; GM key 56 (G#) | mismatch (placement) | — |
| `LowTom`/`MidTom`/`HighTom` | | 808 Low/Mid/Hi Tom | match | — |
| `Conga` (single) on B | (drum.rs:28, :45) | 808 Low/Mid/Hi Conga (switch-selected over toms); GM 62-64 | orcvs-specific | — |
| `Cymbal` on G# | (drum.rs:24, :41) | 808 Cymbal; GM 49 Crash / 51 Ride | mismatch (placement) | — |
| `ClosedHat`/`OpenHat`, choke | (lib.rs:276-279) | 808 Closed/Open Hi Hat; hat choke | match | — |
| `MD` CC1 | Global vibrato (vibrato.rs:1; lib.rs:224) | CC1 Modulation Wheel (GM: vibrato depth, per channel) | acceptable | Note it is global, not per-channel. |
| `OF` CC2 | Start offset (lib.rs:38, :225) | CC2 Breath Controller; "sample start" | mismatch | Enfer convention; document as such. |
| `LN` CC3 | Decay/release scale (lib.rs:52, :226) | CC3 undefined; "decay"/"release" | orcvs-specific | Doc as "release/decay time". |
| `DL` CC4 | Feedback delay level (delay.rs:1-14) | CC4 Foot Controller; "delay send" | mismatch (CC) | GM has no delay CC; any free CC (e.g. 12–31 range) avoids clashing with Foot Controller. Doc warning. |
| `CH` CC5 | Chorus level (chorus.rs:1) | **CC5 Portamento Time**; chorus is CC93 | mismatch | Consider CC93, or warn. |
| `EQ` CC6 | Tilt EQ (tilt.rs:1-12) | **CC6 Data Entry MSB** (RPN/NRPN) | mismatch (risky) | Move off CC6; name "tilt" rather than "EQ". |
| `R1` CC7 | Small reverb level (reverb.rs:1; lib.rs:40-44) | **CC7 Channel Volume**; reverb CC91 | mismatch (serious) | Consider CC91; at least warn in docs/functions.md:1207 that the example is volume elsewhere. |
| `R2` CC8 | Large reverb level | CC8 Balance | mismatch | As above. |
| Reverb implementation | Schroeder 4 comb + 2 allpass, Freeverb tunings (reverb.rs:5-26) | Schroeder/Freeverb | match | — |
| Program Change / Pitch Bend in synth | Ignored (lib.rs:65-67) | — | doc gap | Say so in CONTEXT.md:370. |
| OSC `!o` | One message, no bundle (atom.rs:1326; CONTEXT.md:362) | OSC message | match | — |
| "address", "type string" | (CONTEXT.md:362) | OSC Address Pattern, OSC Type Tag String | acceptable | "type tag string" is the spec phrase. |
| hex-word int32 (`1B8` -> 440) | (CONTEXT.md:362) | — | orcvs-specific | Already documented. |
| UDP `!u` | One datagram (atom.rs:1346) | UDP datagram | match | — |
| Payload | Cells up to closing `!` (CONTEXT.md:366) | message body / payload | acceptable | — |
| Ports 49161 / 49162 | (docs/functions.md network section) | Orca defaults | match (Orca) | — |
