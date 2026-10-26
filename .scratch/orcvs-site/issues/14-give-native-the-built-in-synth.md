# 14 — Give native the Built-in Synth

**What to build:** The native console offers the Built-in Synth as ADR 0087 decides it, playing
ticket 08's synth core through cpal on the console's audio device, with small buffers. It is listed
after the MIDI ports and chosen like one, and plays each message at its Tick's planned time, so the
same Source sounds the same natively and on the site.

**Blocked by:** 08

**Status:** needs-triage

- [ ] Tests through the `MidiBackend` seam that the native destination is listed after the ports
      and that its messages reach the core with their planned times; the core's own tests carry over.
- [ ] A missing audio device answers as a connect failure, and Playback runs on silently.
- [ ] cpal's addition is recorded with its rationale, and `mise run audit_deps` passes.
