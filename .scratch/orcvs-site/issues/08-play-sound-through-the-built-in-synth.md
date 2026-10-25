# 08 — Play sound through the built-in synth

**What to build:** The built-in synth as ticket 07's ADR decides it: a destination the web console
offers beside the Web MIDI ports through `MidiBackend`, whose connection turns Playback's MIDI bytes
into Web Audio voices through `web-sys`. Embed mode selects it. The audio context starts or resumes
on the action that starts Playback. See `../spec.md` (Built-in synth).

**Blocked by:** 02, 07

**Status:** ready-for-agent

- [ ] Tests through the `MidiBackend` seam: the destination is listed beside Web MIDI ports, embed
      mode selects it, and its messages become the voices the ADR specifies, observed at a test-only
      sink below the Web Audio calls. No shipped item takes a parameter only a test supplies.
- [ ] A browser test shows the audio context is running after the Playback gesture.
- [ ] Any `web-sys` feature added is recorded with its rationale, and `mise run audit_deps` passes.
- [ ] `mise run check_wasm` passes.
