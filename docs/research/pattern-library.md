# An Orcvs pattern library

Research dated 17 November 2026 · Design confirmed 18 November 2026 · Orcvs `c8cc197c`

**Agreed release sequence.** Publish Kombine as the first complete library entry, with a playable example, a copyable accent Region, a worked transfer into Tracker, and the checks recorded below. Follow it with preset recall for the Built-in Synth’s acid voice. Each entry demonstrates one meaningful way to grow and appears in Help → Examples with a sibling Markdown guide.

The user confirmed shared understanding of Q1–Q16 on 18 November 2026. These decisions define the agreed first release and second entry. The remaining survey candidates are future possibilities. Implementation and publication checks are pending. The approved delivery tickets are recorded in the [patterns library effort](../../.scratch/pattern-library/spec.md).

## 1. Publish ordinary Source with a guide to copying and scaling it

Start with an index in `examples/` linking a complete `.orcvs` Source File and a sibling Markdown guide for each family. Point to existing files wherever they already demonstrate the construction. A guide should mark the smallest Region worth copying, identify the musical data, give one audible edit, and show a larger form with its expected output. The console registers its examples explicitly, five today, so each entry is added to Help → Examples by hand (Q5). Web discovery remains a separate choice. [Examples](../../examples/) · [Console registry](../../console/src/examples.rs) · [Source Snapshot](../adr/0003-source-snapshot-is-complete-language-state.md)

| Seed | Edit to try | What has to move together |
| --- | --- | --- |
| One Sequencer voice | Extend the Note row, change its Clock `steps`, then add a second voice playing at a different division. Match its Bang cadence if held Notes should sound only once. | Each step occupies two Cells. Copy the reader, trigger, Play and data row. Melody shows the compact Track version. [Sequencer](../../examples/sequencer.md) · [Melody](../../examples/melody.md) |
| One Tracker voice | Shorten one column to eight steps against a sixteen-step neighbour. | A whole-Region translation preserves a relative Read distance. Inserting rows between the Read and the first Note changes that distance. The guide already gives the staircase offsets. [Tracker](../../examples/tracker.md) |
| One Knobs control | Put a cutoff sweep beside a bassline, then add a second controller with a smaller Slew `amount`. | Keep each Slew’s feedback Cells available and route its CC to the sounding channel. Beat supplies a simpler reference for a periodic Bang and a single selected step. [Knobs](../../examples/knobs.md) · [Beat](../../examples/beat.md) |

**Agreed reuse contract (Q1).** Each entry opens as a playable example and marks a Region a musician can copy into an existing piece. The guide identifies the Cells to connect or edit after placement, including any absolute addresses. For a controller-only Region, the complete example supplies a voice so the musician can hear the change.

**Agreed scaling contract (Q2).** Each entry demonstrates one musically meaningful way to grow. The guide identifies the edit and the expected audible result. A Sequencer entry might extend a phrase, a Tracker entry might add a voice, and a Kombine entry might change the accent period independently of pitch.

**Agreed first entry (Q3).** Publish Kombine first. Three pitches cycle against four velocity steps. Shorten the accent cycle to three steps so the loud accent returns to C on each pass. The guide shows both the Clock `steps` and Track `count` edits.

**Agreed worked transfer (Q4).** Demonstrate copying the accent Region into Tracker and connecting it to the velocity operand of one voice. Give the placement, wiring edits and expected accents. The other voices keep their existing behaviour.

**Agreed discovery (Q5).** List playable library entries under Help → Examples and keep each guide beside its Source File. An index in `examples/` links every entry, and each registered entry keeps the web link name registration gives it (both confirmed 18 November 2026).

**Agreed accent timing (Q6).** In the Tracker transfer, drive the accent Clock from absolute Tick with a division of two. The closed hi-hat samples four successive velocities on its four successive scheduled steps. Deleting a hit leaves a rest and preserves the phase of subsequent accents.

**Agreed copyable Region (Q7).** Copy the accent Clock, Track and velocity data together. Mark their output pair and include space for the Cells they write. The guide connects that output to the closed hi-hat’s velocity operand.

**Agreed publication checks (Q8).** Verify Kombine’s twelve-Tick pitch and velocity cycle, the three-step edit that fixes the loud accent on C, and all four velocities in the Tracker transfer. Follow the published transfer instructions in the console and listen on the Built-in Synth. These checks are acceptance criteria for the future entry.

**Agreed first release (Q9, 18 November 2026).** Publish the library with one complete Kombine entry after its standalone example, worked Tracker transfer, guide and listening check pass. Use the first transfer’s results to refine subsequent entries.

**Agreed Region documentation (Q10).** A diagram gives the copyable Region’s two corners, its output pair and the destination in Tracker. Include blank Cells used for runtime writes. A short `Accent` Comment above the Region identifies it in the playable Source. Every entry's guide carries such a diagram of its Region (confirmed 18 November 2026).

**Agreed second entry (Q11).** Follow Kombine with preset recall. One selection changes several controller targets, each approaching its target at its own Slew `amount`.

## 2. Assemble one voice from independent pitch, accent and rhythm rows

The first study below proves a three-pitch, four-velocity voice with a moving accent. Orca’s *Kombine* uses separate octave, note and velocity clocks. Monome’s Kria extends that idea to separately looped parameters. Start with ordinary Orcvs Notes so the musician can hear the effect of changing one row length. [Kombine](../reference/learn-orca/examples/misc/kombine.orca) · [Kria](https://monome.org/docs/ansible/kria/)

**Future scale test, outside the agreed first release.** Give one voice seven pitches, five velocities and a sixteen-step numeric mask. Advance each row once per Tick and emit a Timed Play only on a hit. All three positions return together after 560 Ticks. Repeated data can make the audible phrase shorter. A rest skips a new onset while the previous Note keeps its scheduled lifetime. Check Tick zero, each row’s wrap, the first shared wrap and a live edit to one row. A second voice reading the same pitch data with its own accents would demonstrate reuse. Four voices would test a larger layout. This masked and shared version still needs a Grid and an output trace. [Track, Equality and Timed Play](../functions.md) · [Playback tests](../../orcvs/src/playback/tests/shipped_trackers.rs)

**For a later masked voice:** With mask `1001` and pitches C, D, E, G, should the two hits play **C, G** (time advances through rests) or **C, D** (only hits advance)? The recommendation is elapsed Ticks for the first entry. Counting accepted hits needs a separate state transition and a defined first hit. Decide whether a second voice must share live pitch edits or can start with a private copy. [Tidal struct](https://userbase.tidalcycles.org/struct.html) · [Orcvs Function reference](../functions.md)

## 3. Recall a bank of controller settings with independent slews

Knobs already automates its targets. Its Orca ancestor recalls target settings. The second study below demonstrates one CC and two targets. The library entry extends this to a shared selector and separate feedback for each control. [Orca Knobs](../reference/learn-orca/examples/setups/knobs.orca) · [Orcvs Knobs](../../examples/knobs.md) · [Control playback tests](../../orcvs/src/playback/tests/shipped_knobs.rs)

**Agreed live selection (Q12).** The musician edits one selector. Changing `00` to `01` changes every controller’s target on the next Tick.

**Agreed transition behaviour (Q13).** A new selection during a slew redirects each controller from its current value toward the new target. Each retains its fixed Slew `amount`, so completion time depends on distance.

**Agreed size and growth (Q14).** Start with two presets controlling cutoff and resonance on the Built-in Synth’s acid voice. Demonstrate scaling by adding decay as a third control driven by the same selector while the bassline plays.

**Agreed initialization and restart (Q15).** Open with preset `00` established in the control values. Switching to `01` demonstrates the slew. Rewind keeps the Source’s Cell values, so the controls stay where the Source holds them. No ADR or test states this: [ADR 0076](../adr/0076-stopping-playback-holds-its-tick.md) settles only that a Rewind makes Tick `0` the next Tick, and ticket 02 requires the test that holds it. Reopening the example restores its initial settings.

**Agreed selector range (Q16).** Track wraps the numeric selector by preset count. With two presets, `02` selects the first and `03` the second. Document the MIDI target range and use existing diagnostics for malformed input.

The completed entry should exercise initialization, reopening, rewind, selector wrap, a mid-slew change, a zero Slew `amount` (hold), a stable target, and delivery on the intended channel. [Slew](../functions.md) · [Rewind and the held Tick](../adr/0076-stopping-playback-holds-its-tick.md)

## 4. Vary a phrase by changing which step plays

The fourth study gives the first arrangement entry: a slow counter chooses a fill for a whole phrase. In that Grid a phrase lasts four Ticks, one beat at four Ticks per beat. Tidal’s cycle conditions inspire the choice. Orcvs can express this particular choice with an index and a table. Whether a time transformation fits depends on whether its required onsets fall on Orcvs Ticks. [Tidal conditions](https://tidalcycles.org/docs/reference/conditions/) · [Orcvs timing](timing-operators.md)

**Future choice after preset recall.** Which edit should the musician learn next: write an alternate phrase, move the start of one phrase against another instrument, or keep the notes and change their rhythm? Use the fourth-phrase fill as the reference. The following candidates each add a different audible result. Choose one for a playable follow-up.

| Candidate | Smallest demonstration and open question |
| --- | --- |
| Rotate or slide a read head | Tidal’s `iter` shifts a four-note phrase by one step per pass. SuperCollider’s `Pslide` takes three-note windows: C D E, D E G, E G C. Orca’s back-and-forth Track uses a six-step path over four notes. Which edit should the user control: rotation, window stride, or direction at an endpoint? [Tidal iter](https://tidalcycles.org/docs/reference/accumulation/#iter) · [SuperCollider Pslide](https://doc.sccode.org/Classes/Pslide.html) · [Orca example](https://git.sr.ht/~rabbits/orca-examples#play-a-sequence-back-and-forth) |
| Interlock two drum voices | Give one voice the three-in-eight rhythm `X..X..X.` and the other its complement. All eight steps should have exactly one onset. An editable binary mask also supports Orca-style AND, OR and XOR. Decide whether the performer edits hits or a table. The shipped Euclidean takes hits and steps. [Orca gates](../reference/learn-orca/examples/misc/gates.orca) · [Tidal euclidFull](https://userbase.tidalcycles.org/euclidFull.html) · [Euclidean](../functions.md) |
| Play harmony or choose tones | The third study samples a triad on sparse hits. A chord tracker reads a root and Interval Set per step and sounds several tones together. Choose whether editing the root changes a melody or a whole voicing. A later weighted-choice variant can index four entries C, C, C, G with Random for a nominal 3:1 weighting. [Orca chord](../reference/learn-orca/examples/misc/chord.orca) · [Chord tracker test](../../orcvs/src/source/tick/driven_chord.rs) · [SuperCollider pattern guide](https://doc.sccode.org/Tutorials/A-Practical-Guide/PG_02_Basic_Vocabulary.html) |
| Traverse a two-dimensional score | Orca’s Popcorn and Make Noise’s René suggest selecting a note with separate X and Y clocks. For packed pairs the address is `(first column + 2x, first row + y)`. Show the first visited positions and test the Grid edge before promising relocation. [Popcorn](../reference/learn-orca/examples/misc/popcorn.orca) · [René manual](https://www.makenoisemusic.com/wp-content/uploads/2024/03/renemanual.pdf) |

**Hold for a concrete musical need.** Orca’s Wave times a strike by a mover’s flight and collision. A recorded random loop needs controlled capture, retention and replay. Both deserve a Tick-by-Tick demonstration of writes, activation and restart. Sub-Tick ratchets would need timing evidence beyond these whole-Tick grids. New Functions, editor transforms and a generator can be evaluated against a specific patch that proves hard to express or copy. [Wave](../reference/learn-orca/examples/misc/wave.orca) · [Movement and Push](../functions.md) · [Timing proposals](timing-operators.md)

## What would count as a library entry?

The two tickets state the acceptance criteria for the agreed entries. For later entries, the proposed publication evidence is to show the minimal and scaled Source, the Cells the musician edits, the space read or written at runtime, and the expected events at startup and wrap. State which values are shared and how to move the example. Exercise the actual file after one advertised edit and check emitted Notes or CCs, routing, and diagnostics. For a live-performance claim, also try the edit during Playback and listen on the named instrument. The existing tracker and knob tests provide models for checking delivery. [Tracker tests](../../orcvs/src/playback/tests/shipped_trackers.rs) · [Knobs tests](../../orcvs/src/playback/tests/shipped_knobs.rs)

## Evidence and provenance

This consolidation covers the repository note and two documents supplied outside the repository, *Orcvs patterns library research.md* and *orcvs-patterns-library-agent-handoff-2026-11-17.md*. The handoff cites a separate, longer catalogue at revision `12bb62c6`. The families here are grouped by the edit a musician can make. The current checkout supplies the Function spellings, example registry and execution evidence. The larger designs above remain candidates for later design. [Function reference](../functions.md) · [Learn Orca example provenance](../reference/learn-orca/README.md)

## Four grids you can try

Each block is a separate grid. Paste it at the top-left of an empty Source, preserving indentation and blank rows. All four ran for 32 consecutive Ticks from Tick 0 on Orcvs revision `c8cc197c` without execution diagnostics. That run was a one-off: no test holds these Grids, and the linked test shows only how Source was executed for it. The observations below are from emitted commands. The two examples using `&$` address absolute Cells, so moving the Region requires updating their Positions. [Function reference](../functions.md) · [Source execution used for the check](../../orcvs/tests/function_reference_doc.rs)

### Independent pitch and accent lanes

```orcvs
~*0101
  !~00&$0006&$000401

&t~.010303C4E4G4

&t~.01040470403040
```

Listen for the accent moving across the triad. The upper Track cycles through C4, E4 and G4. The lower one supplies four hexadecimal velocities. The loud `70` lands on C4 at Tick 0, E4 at 4, G4 at 8 and C4 again at 12. The Play reads velocity from row 6 and pitch from row 4. Orca's *Kombine* assembles octave, note and velocity at three clock divisions. This version folds octave into the Note and varies the lane lengths. For the original's slower pitch changes, change the pitch Clock from `~.0103` to `~.0203`. [Kombine grid](../reference/learn-orca/examples/misc/kombine.orca) · [Orcvs Track and Timed Play](../functions.md)

### Two filter presets with a slew

```orcvs
~*0101
  !c084A~>10&t~.0402022040
```

Run alongside a bassline on channel `08` to hear the filter. This grid sends controller commands on its own. Track chooses cutoff targets `20` and `40`, holding each for four Ticks. From a blank start, Slew sends `10 20 20 20 | 30 40 40 40 | 30 20 …` (hex). Change the Slew `amount` from `10` to `20` to reach the next preset in one Tick. For a bank of scenes, duplicate the lane with different targets and CC numbers, leaving their Clocks identical so the settings change together. [Original Knobs grid](../reference/learn-orca/examples/setups/knobs.orca) · [Orcvs Knobs](../../examples/knobs.md)

### A sparse rhythm sampling a triad

```orcvs
~%0308
  !~0064#pC4MA~.010301
```

The three-in-eight rhythm samples a three-step pitch Clock. The first eight Ticks sound C4 three times (Ticks 0, 3 and 6), the next eight sound G4 three times, and the next eight sound E4 three times. Both cycles start together again at Tick 24. This distinction matters when adapting an arpeggio: the Clock follows elapsed Ticks, so rests still advance its phase. Change the gate to `~*0101` to hear C4, E4, G4 successively. [Orca arpeggio](../reference/learn-orca/examples/misc/arpeggio.orca) · [Orcvs Euclidean, Clock and Pick](../functions.md)

### A fill every fourth phrase

```orcvs
~*0101
  !~0064&$000401

&t.+~.0104.x04./~.04040308C4  G4  C4D4E4G4
```

Read the eight pairs at the right as two phrases: `C4 · G4 ·` and `C4 D4 E4 G4`, where each dot represents two blank Cells. The first phrase plays three times. Ticks 12–15 play the fill, then Tick 16 returns to the first phrase. The fast Clock gives the step within the phrase. The slow Clock counts phrases 0–3. Integer division by three and multiplication by four give a bank offset of 0, 0, 0 or 4. Add the step and Track has its index. The separate output row leaves the Track’s data clear of nested Functions' writes. Edit just the last four pairs to replace the fill. This borrows Tidal's conditional variation and explicitly chooses the **fourth** phrase as the fill. [Tidal cycle conditions and phase offsets](https://tidalcycles.org/docs/reference/conditions/) · [Orcvs arithmetic and Track](../functions.md)
