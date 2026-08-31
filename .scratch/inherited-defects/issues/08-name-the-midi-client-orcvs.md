# 08 — Name the MIDI client Orcvs

**What to fix:** The native MIDI backend registers itself as "Orca". The project is Orcvs. The name
is written twice, so the two copies can also drift apart.

**Status:** ready-for-agent

- [ ] The MIDI client name matches the project name.
- [ ] The name is written once.
- [ ] The port name is consistent with the client name.

## Comments

`orcvs/src/native_midi.rs:14` and `:29` both call `MidiOutput::new("Orca")`. Line 35 opens the port
as `"Orca output"`.

A digital audio workstation shows the client name in its MIDI routing list. A user who looks for
Orcvs finds an entry called Orca, which is a different program.

Give the module one constant and use it in all three places. `MidiOutput::new` is called in
`destinations` and again in `connect`, so the literal appears twice for a structural reason, not by
accident.

CONTEXT.md gives the vocabulary. The unqualified system name is Orcvs.
