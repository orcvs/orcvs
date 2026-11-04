# 20 — Play the chorus effect

**What to build:** The Built-in Synth plays `CH`, CC 5, as ADR 0088 decides it. A chorus on the whole mix. `00` is dry, and higher values deepen it.
The Sound Module already shows and sets the value (ticket 15), and its load value leaves the synth
sounding as it does with no effects.

**Blocked by:** 15

**Status:** resolved

- [x] Synth core tests: at the load value the sound is unchanged, and across the range the effect is
      audible and stays finite.

## Answer

Two delay taps around 12 ms that swing 4 ms either way at 0.7 Hz in opposite phase, mixed into the whole mix after `MD` and before `DL`, up to half and half at `7F`. The level glides over at most 20 ms.
