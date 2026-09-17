# 06 — Prove the reference is complete

**What to build:** A test that fails when a Function exists in the Function table but has no example in the reference, so adding a Function without documenting it breaks the build.

**Blocked by:** 02 — Add the Conversion and Sequence Functions; 03 — Add the Tick Functions; 04 — Add the Jump, Bang and movement Functions; 05 — Add the MIDI output Functions.

**Status:** ready-for-agent

- [ ] The test reads the Function table and the reference's Language Map, and asserts every Function spelling appears as the root of at least one example.
- [ ] Its failure message names the missing Functions.
- [ ] Every example Expression in the reference parses without diagnostics (result rows excepted).
