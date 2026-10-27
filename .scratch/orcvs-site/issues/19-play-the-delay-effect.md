# 19 — Play the delay effect

**What to build:** The Built-in Synth plays `DL`, CC 4, as ADR 0088 decides it. An echo of the whole mix. `00` is dry, and higher values bring the echo up.
The Sound Module already shows and sets the value (ticket 15), and its load value leaves the synth
sounding as it does with no effects.

**Blocked by:** 15

**Status:** needs-triage

- [ ] Synth core tests: at the load value the sound is unchanged, and across the range the effect is
      audible and stays finite.
