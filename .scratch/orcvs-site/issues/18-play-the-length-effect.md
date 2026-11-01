# 18 — Play the length effect

**What to build:** The Built-in Synth plays `LN`, CC 3, as ADR 0088 decides it. `LN` scales how long each sound decays, and a tone's release, from full length at `7F` down to a short blip at `00`.
The Sound Module already shows and sets the value (ticket 15), and its load value leaves the synth
sounding as it does with no effects.

**Blocked by:** 15

**Status:** resolved

- [x] Synth core tests: at the load value the sound is unchanged, and across the range the effect is
      audible and stays finite.

## Answer

Each new sound takes a length from `LN` when it starts: exactly 1 at `7F`, falling exponentially to 0.02 at `00`. A drum's loudness decays on a clock that runs `1 / length` times as fast while its pitch keeps real time, a tone's 150 ms release is scaled by it, and the acid voice's filter envelope decays in that share of its time. At `00` a kick is a blip of under 10 ms.
