# 04 — Send Control Change and Pitch Bend

**What to build:** Implement fixed-arity terminal Functions `!c channel controller value` and
`!b channel lsb msb`, preserving direct MIDI wire bytes.

**Blocked by:** 01 — Generalize Play Commands for MIDI output.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Control Change emits the correct status, controller, and value bytes.
- [ ] Pitch Bend emits the correct status, LSB, and MSB bytes.
- [ ] Channel accepts `00`–`0F`; every data byte accepts `00`–`7F`.
- [ ] Invalid operands diagnose and emit no command.
- [ ] No scaling, normalization, wrapping, or clamping occurs.
- [ ] Multiple commands retain Tick Plan and output-adapter order.
- [ ] Native delivery and in-memory adapter tests assert exact byte sequences.
