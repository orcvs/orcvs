# 05 — Add the MIDI output Functions

**What to build:** The reference gains a MIDI group with an example of each Terminal Output Function, each with a Bang source so it sends when played.

**Blocked by:** 01 — Open on a Function reference, starting with the Arithmetic Functions.

**Status:** ready-for-agent

- [ ] One example each for `!>`, `!~`, `!%`, `!c`, `!b`, with operands that name a plausible channel, note, velocity, controller or bend.
- [ ] Each has a Bang source that fires at a rate slow enough to hear distinct events at the default tempo.
- [ ] No result row: a Terminal Output Function writes nothing, and the example says so in its layout (no row reserved south).
- [ ] A test proves ticking the group writes no Cell.
