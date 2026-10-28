# 18 — Play the length effect

**What to build:** The Built-in Synth plays `LN`, CC 3, as ADR 0088 decides it. `LN` scales how long each sound decays, and a tone's release, from full length at `7F` down to a short blip at `00`.
The Sound Module already shows and sets the value (ticket 15), and its load value leaves the synth
sounding as it does with no effects.

**Blocked by:** 15

**Status:** needs-triage

- [ ] Synth core tests: at the load value the sound is unchanged, and across the range the effect is
      audible and stays finite.
