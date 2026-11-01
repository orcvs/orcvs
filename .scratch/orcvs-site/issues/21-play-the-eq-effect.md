# 21 — Play the eq effect

**What to build:** The Built-in Synth plays `EQ`, CC 6, as ADR 0088 decides it. One tilt on the whole mix. `40` is flat, lower values are darker and higher values are brighter.
The Sound Module already shows and sets the value (ticket 15), and its load value leaves the synth
sounding as it does with no effects.

**Blocked by:** 15

**Status:** resolved

- [x] Synth core tests: at the load value the sound is unchanged, and across the range the effect is
      audible and stays finite.

## Answer

A one-pole lowpass splits the mix at 800 Hz. The tilt raises one side by up to 12 dB and lowers the other by as much: `00` is darkest, `40` is exactly flat, `7F` is brightest. It runs after `MD` and before `CH`, and the tilt glides rather than steps.
