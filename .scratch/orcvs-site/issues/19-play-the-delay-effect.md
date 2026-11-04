# 19 — Play the delay effect

**What to build:** The Built-in Synth plays `DL`, CC 4, as ADR 0088 decides it. An echo of the whole mix. `00` is dry, and higher values bring the echo up.
The Sound Module already shows and sets the value (ticket 15), and its load value leaves the synth
sounding as it does with no effects.

**Blocked by:** 15

**Status:** resolved

- [x] Synth core tests: at the load value the sound is unchanged, and across the range the effect is
      audible and stays finite.

## Answer

A feedback delay on the whole mix after `MD`: echoes every 375 ms (three Ticks at the default tempo), 45 % feedback through a 3 kHz lowpass so each echo is darker, added to the mix at `DL`'s level. The level glides over at most 20 ms, so a change mid-echo does not click.
