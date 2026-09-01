# 01 — Generalize Play Commands for MIDI output

**What to build:** Replace the single raw-note command shape with explicit ordered MIDI command
variants capable of carrying ADR 0016's terminal outputs without leaking MIDI byte assembly into
Source interpretation.

**Blocked by:** orcvs-language-migration/02; orcvs-language-migration/04; lang-foundations/06.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Raw Play uses `!> channel velocity note` and emits only when its root is active.
- [ ] Channel, velocity, Note, and later data-byte domains diagnose before command creation.
- [ ] Terminal Functions remain invalid as nested value-producing Functions.
- [ ] Tick Plan retains command producer order.
- [ ] Tick planning, not MIDI interpretation, owns whether a root is active; integration coverage
      proves inactive terminal roots emit nothing and active roots retain Tick Plan order.
- [ ] Playback/output adapter seam accepts explicit commands without parsing Source intent.
- [ ] Existing raw Note On delivery and velocity-zero behavior remain covered.
