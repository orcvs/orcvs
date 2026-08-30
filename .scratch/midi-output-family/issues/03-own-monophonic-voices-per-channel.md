# 03 — Own Monophonic voices per channel

**What to build:** Implement `!% channel velocity note length` with one Mono-owned voice per output
adapter and MIDI channel.

**Blocked by:** 02 — Schedule Timed Play Note Off.

**Status:** ready-for-agent

- [ ] Every command stops the prior Mono-owned note on its channel first.
- [ ] Velocity `00` or length `00` replaces ownership with silence and starts nothing.
- [ ] Positive commands own the replacement and schedule Note Off at Tick `T + length`.
- [ ] Generation tokens prevent stale expiry from stopping a replacement.
- [ ] Channels are independent and the last same-Tick command owns its channel.
- [ ] Raw and Timed Play notes never enter Mono ownership.
- [ ] Stop and adapter lifecycle safety clear every Mono voice.
