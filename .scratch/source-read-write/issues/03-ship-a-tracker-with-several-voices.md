# 03 — Ship a tracker with several voices

Status: ready-for-agent
Blocked by: 02 — Read a pair in a direction; 07 — Accept the Read, Write and Copy families; cell-tracker/05 — Verify live editing in the console

**What to build:**

Port the Orca TRACKER patch: one Clock shared by several voices, each offsetting it with `.+` and triggering its own Timed Play. Each voice's `&v` sits at the head of its own column of a pattern grid and reads down it. `&v 00` is the Read's own operand, so each voice offsets its row index by one, for example Clock → `.+ 01` → `&v`, and its column begins on the row directly below the operand, beside the Read's Output Portal. Ship the Source File and a guide, loadable through the console's File > Open.

## Acceptance criteria

- [ ] The shipped Source File has at least four voices, each with its `&v` at the head of its own column, with some deliberately empty steps.
- [ ] The guide maps each Orca operator in the patch to its Orcvs Function, explains that `&v` counts rows from its own operand and why each voice adds `01`, and gives the BPM, MIDI channels, velocity and length, with exact columns for spatial alignment.
- [ ] A test loads the exact shipped Source File and checks two complete loops of every voice through Source/Tick and Playback, including empty steps and Note Off timing, asserting each Tick's diagnostics exactly.
- [ ] The example works through the console editing and file workflows verified by `cell-tracker/05`: edit a step, clear and restore it, Save and reopen.
- [ ] Record an audible MIDI smoke test with device, channels and BPM; do not mark delivery complete on automated evidence alone.
