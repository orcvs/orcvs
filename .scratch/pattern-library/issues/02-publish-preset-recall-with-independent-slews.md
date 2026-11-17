# 02 — Publish preset recall with independent slews

Status: ready-for-human
Blocked by: 01

**What to build:** A musician opens a bassline with two presets from Help →
Examples and edits one selector to slew cutoff and resonance together. The
guide demonstrates adding decay as a third control while the bassline plays.
Use the entry format and transfer lessons established by ticket 01.

- [x] Ship a playable Source File and sibling guide, register the entry under
      Help → Examples, and add it to the library index.
- [x] Provide two presets controlling cutoff and resonance on the Built-in
      Synth's acid voice on channel `08`. Document the selector, target values,
      per-control Slew `amount` operands, CC mapping, channel and update cadence.
- [x] Start with preset 00 established in each control's Source value and
      deliver those values to the instrument. Changing the selector to 01
      redirects every target on the next Tick.
- [x] Give each control independent current-value feedback and a fixed Slew
      `amount`. Switching presets mid-slew continues from the current value
      and each control's completion time depends on its distance. Verify the
      limit of `amount` each Tick, eventual arrival at a stable target and
      intentional holding when `amount` is zero.
- [x] Wrap numeric selection by preset count. Verify 02 selects the first
      preset and 03 the second. Document the MIDI target range and preserve
      existing diagnostics for malformed input.
- [x] Verify that Rewind retains the current Source control values and reopening
      restores the example's initial preset. No ADR or test states the first
      of these (ADR 0076 settles only the Tick), so add the test that holds it.
      Explain these behaviours in the guide.
- [x] Mark a copyable control Region with a diagram covering the Region's
      corners, runtime writes, shared selector references and output
      mapping. Explain placement and any address edits.
- [x] Demonstrate and verify adding decay as a third control driven by the same
      selector. Decay is controller 75 (`4B`) on channel `08`; a Note at velocity
      120 (`78`) or above is accented and takes the shortest decay whatever
      the controller says (`an_accented_note_takes_the_shortest_decay` in
      `synth/src/lib.rs`). The audible result is a shorter or longer decay
      on unaccented Notes only, which the bassline's velocities must allow,
      and the guide says so. Preserve the bassline
      cadence and the existing controls' mappings and slew behaviour.
- [ ] Exercise the actual shipped Source and scaling edit through existing
      playback testing conventions, checking delivered CCs and diagnostics.
      Follow the guide in the console and listen on the Built-in Synth.
- [ ] Complete the repository gates applicable to the changes and record
      automated, console and listening evidence distinctly.

## Comments

**17 November 2026 — implementation on
`claude/project-thread-4dv2du-preset-recall`.** Eight of the ten boxes are
done. The box that asks for following the guide in the console and listening,
and the evidence box, stay open until a person follows the guide in the
console and listens on the Built-in Synth.

The entry is **Presets**: `examples/presets.orcvs` and `examples/presets.md`,
listed under Help → Examples with link name `presets` and in the examples
index and its "Patterns to copy" table.

Design decisions a reviewer should check:

- The selector wraps with Modulo, `.%0002`, rather than Track. A Track's data
  pairs are ordinary Source the Language Map diagnoses (Kombine opens that
  way). Keeping the presets in Comment text, read by a Read south whose
  distance an Add computes from the preset in force, lets the Source open,
  and stay on every Tick, with nothing diagnosed. Q16 says "Track wraps the
  numeric selector"; the ticket box says only "wrap numeric selection by
  preset count".
- Each control's Slew feedback, and every value a nested Function writes, is
  Comment text. A Slew reads and writes Comment text as it does any Cells, so
  preset `00` ships in each control's value and the first Tick delivers it.
- The decay control is added by copying the `CUT` Region, the copyable
  Region the guide marks. The guide also moves the Preset block and both
  controls into the Tracker, whose bassline is on channel `08` at velocity
  `77`, to work the address edit through.

Automated evidence:

- `orcvs/src/playback/tests/shipped_presets.rs` runs the shipped file through
  `HandDrivenRun`, with each expected value written out Tick by Tick, the
  number of delivered command lists checked, and every edited Source built
  from the shipped file by `replace_at`. It holds: nothing diagnosed at load
  or on any Tick, and every shipped row drawn by the guide; preset `00`
  delivered from Tick 0 for two bassline loops; the selector to `01`
  turning both controls on the next Tick, `CUT` by `08` to `60` in 8 Ticks
  and `RES` by `05` to `50` in 10 with a last step of `03`, then holding;
  a change back mid-move turning each from its current value, `CUT` in 8
  Ticks and `RES` in 9; `amount` `00` holding through a preset change;
  `02`, `03` and `FF` wrapping; `0G` and an emptied selector Cell
  diagnosed on the Modulo with the existing operand diagnostics while the
  preset in force stays; a target past `7F` diagnosed on the Control Change
  (`MIDI value 80 is outside the range 00–7F`) and silencing only that
  control; a stopped and Rewound run keeping every Cell and resuming each
  control from its held value; the decay control drawn as the guide draws
  it, diagnosed nowhere, following the selector while the bassline, cutoff
  and resonance deliver exactly as before; and the move into the Tracker,
  which adds no diagnosis and leaves every voice's Notes.
- Whether a nested Slew advances when its Control Change is not banged is
  settled by `a_control_banged_every_second_tick_moves_only_when_it_sends`:
  it does not. The guide states the update cadence accordingly.
- `orcvs/src/playback.rs` `a_rewind_keeps_the_cells_the_runs_before_it_wrote`
  drives the Playback Engine: a Rewind, stopped or live, changes no Cell,
  and a Slew carries on from the value the run before left. No test held
  this before.

Console evidence (automated in a headless harness, not by a person), in
`console/src/console/kittest_tests/live_editing.rs`:

- `a_preset_typed_in_the_console_turns_both_presets_controls` opens Presets
  from Help → Examples and types `1` at (5, 15).
- `the_presets_decay_control_pasted_in_the_console_follows_the_selector`
  copies the `CUT` Region with Shift-arrows, pastes the Copy's clipboard
  text at (0, 30) and types the guide's five edits.
- `the_presets_controls_pasted_into_the_tracker_slew_its_bassline` follows the
  move into the Tracker.
- `reopening_the_presets_restores_preset_00` opens Presets again after a
  preset change: the console asks, and Discard restores the shipped Source.

Listening: none. No one has listened to this entry, and no offline render
was made.

Remaining for a person:

- Open Presets in the console and play it on the Built-in Synth. Change the
  selector, add decay as the guide says, and listen. Check that preset `01`
  is audibly brighter and sharper, that the two controls' different speeds
  can be heard, and that decay `10` against `70` is audible on the
  unaccented Notes.
- Record what you hear, and fix the guide or Source wherever the result
  differs. The guide's descriptions of the sound are not yet confirmed by
  listening.
