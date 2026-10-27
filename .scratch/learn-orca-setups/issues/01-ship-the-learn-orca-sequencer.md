# 01 — Ship the Learn Orca sequencer

Status: resolved

**What to build:**

Port the SEQUENCER patch from Learn Orca's setups menu
(`docs/reference/learn-orca/examples/setups/sequencer.orca`) as `examples/tracker.orcvs` ports
its TRACKER: ten voices, each reading a thirty-two-step row of Notes and playing it on its own MIDI
channel, with a guide and an entry in `Help > Examples`.

## Acceptance criteria

- [x] `examples/sequencer.orcvs` has ten voices, each with its steps in one closed Comment, empty
      as Learn Orca ships them.
- [x] `examples/sequencer.md` maps each Orca operator in the patch to its Orcvs Function, gives the
      channels, velocity, length and BPM with exact columns, and lists where it differs from Orca.
- [x] A test runs the exact shipped file for two loops, silent as shipped and with a Note in each
      voice, asserting every Tick's delivery and diagnostics.
- [x] `Help > Examples > Orca Sequencer` opens it, and `/play/?example=orca-sequencer` links to it.

## Comments

### 2026-10-27 — Resolved

Each voice is a Delay `~*0101` and an Add `.+04~.0120` over a Timed Play `!~0v77&>  01`, whose
nested Read east reads the step `04 + step` pairs east of its `n` slot. Orca's tenth voice plays on
channel `a` and none on `9`; the port uses `00` to `09`. Tests:
`playback::tests::shipped_trackers::the_sequencer_ships_with_every_step_resting`,
`every_sequencer_voice_plays_its_own_row_on_its_own_channel` and
`the_sequencer_frames_each_voices_steps_in_a_comment`. The audible check is `03`.
