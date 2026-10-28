# 22 — Play the reverb 1 effect

**What to build:** The Built-in Synth plays `R1`, CC 7, as ADR 0088 decides it. The smaller of two reverbs on the whole mix. `00` is dry.
The Sound Module already shows and sets the value (ticket 15), and its load value leaves the synth
sounding as it does with no effects.

**Blocked by:** 15

**Status:** needs-triage

- [ ] Synth core tests: at the load value the sound is unchanged, and across the range the effect is
      audible and stays finite.
