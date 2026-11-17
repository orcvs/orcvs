# 02 — Publish preset recall with independent slews

Status: claimed
Blocked by: 01

**What to build:** A musician opens a bassline with two presets from Help →
Examples and edits one selector to slew cutoff and resonance together. The
guide demonstrates adding decay as a third control while the bassline plays.
Use the entry format and transfer lessons established by ticket 01.

- [ ] Ship a playable Source File and sibling guide, register the entry under
      Help → Examples, and add it to the library index.
- [ ] Provide two presets controlling cutoff and resonance on the Built-in
      Synth's acid voice on channel `08`. Document the selector, target values,
      per-control Slew `amount` operands, CC mapping, channel and update cadence.
- [ ] Start with preset 00 established in each control's Source value and
      deliver those values to the instrument. Changing the selector to 01
      redirects every target on the next Tick.
- [ ] Give each control independent current-value feedback and a fixed Slew
      `amount`. Switching presets mid-slew continues from the current value
      and each control's completion time depends on its distance. Verify the
      limit of `amount` each Tick, eventual arrival at a stable target and
      intentional holding when `amount` is zero.
- [ ] Wrap numeric selection by preset count. Verify 02 selects the first
      preset and 03 the second. Document the MIDI target range and preserve
      existing diagnostics for malformed input.
- [ ] Verify that Rewind retains the current Source control values and reopening
      restores the example's initial preset. No ADR or test states the first
      of these (ADR 0076 settles only the Tick), so add the test that holds it.
      Explain these behaviours in the guide.
- [ ] Mark a copyable control Region with a diagram covering the Region's
      corners, runtime writes, shared selector references and output
      mapping. Explain placement and any address edits.
- [ ] Demonstrate and verify adding decay as a third control driven by the same
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
