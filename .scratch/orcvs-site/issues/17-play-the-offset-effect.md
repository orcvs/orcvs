# 17 — Play the offset effect

**What to build:** The Built-in Synth plays `OF`, CC 2, as ADR 0088 decides it. Each new sound starts up to 100 ms into its course at `7F`, which softens its attack. At `00` it starts at the beginning.
The Sound Module already shows and sets the value (ticket 15), and its load value leaves the synth
sounding as it does with no effects.

**Blocked by:** 15

**Status:** resolved

- [x] Synth core tests: at the load value the sound is unchanged, and across the range the effect is
      audible and stays finite.

## Answer

The synth plays each new sound's first frames unheard, up to 100 ms at `7F`, then fades it in over 5 ms so a sound joined at full loudness does not click. A sound that ends within the offset, such as a rim shot at `7F`, takes no voice. A slide on the acid channel is not a new sound, so it is not offset.
