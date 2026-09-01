# 01 — Generalize Play Commands for MIDI output

**What to build:** Replace the single raw-note command shape with explicit ordered MIDI command
variants capable of carrying ADR 0016's terminal outputs without leaking MIDI byte assembly into
Source interpretation.

**Blocked by:** orcvs-language-migration/02; orcvs-language-migration/04; spatial-tick-planning/02; lang-foundations/06.

**Status:** ready-for-agent

- [ ] Raw Play uses `!> channel velocity note` and emits only when its root is active.
- [ ] Channel, velocity, Note, and later data-byte domains diagnose before command creation.
- [ ] Terminal Functions remain invalid as nested value-producing Functions.
- [ ] Tick Plan retains command producer order.
- [ ] Playback/output adapter seam accepts explicit commands without parsing Source intent.
- [ ] Existing raw Note On delivery and velocity-zero behavior remain covered.
