# 02 — Publish preset recall with independent glides

Status: ready-for-agent
Blocked by: 01

**What to build:** A musician opens a bassline with two presets from Help →
Examples and edits one selector to glide cutoff and resonance together. The
guide demonstrates adding decay as a third control while the bassline plays.
Use the entry format and transfer lessons established by ticket 01.

- [ ] Ship a playable Source File and sibling guide, register the entry under
      Help → Examples, and add it to the library index.
- [ ] Provide two presets controlling cutoff and resonance on the Built-in
      Synth's bass voice. Document the selector, target values, per-control
      slew amounts, CC mapping, channel and update cadence.
- [ ] Start with preset 00 established in each control's Source value and
      deliver those values to the instrument. Changing the selector to 01
      redirects every target on the next Tick.
- [ ] Give each control independent current-value feedback and a fixed slew
      amount per evaluation. Switching presets mid-glide continues from the
      current value. Verify the per-Tick limit, eventual arrival at a stable
      target and intentional holding when the slew amount is zero.
- [ ] Wrap numeric selection by preset count. Verify 02 selects the first
      preset and 03 the second. Document the MIDI target range and preserve
      existing diagnostics for malformed input.
- [ ] Verify that Rewind retains the current Source control values and reopening
      restores the example's initial preset. Explain these behaviours in the guide.
- [ ] Mark a copyable control Region with a coordinate diagram covering
      selection corners, runtime writes, shared selector references and output
      mapping. Explain placement and any address edits.
- [ ] Demonstrate and verify adding decay as a third control driven by the same
      selector. Preserve the bassline cadence and the existing controls'
      mappings and glide behaviour.
- [ ] Exercise the actual shipped Source and scaling edit through existing
      playback testing conventions, checking delivered CCs and diagnostics.
      Follow the guide in the console and listen on the Built-in Synth.
- [ ] Complete the repository gates applicable to the changes and record
      automated, console and listening evidence distinctly.
