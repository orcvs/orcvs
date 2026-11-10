# 02 — Add the Interval Set and Voicing literals

Status: resolved

Blocked by: None — can start immediately

**What to build:** ADR 0105. Interval Set and Voicing become Atoms, decoded by operand position and
carried by Copy, Read, Track, Write and Push. No Function takes them yet, so this slice proves
decoding and carrying.

## Acceptance criteria

- [x] All 39 Interval Set names decode to the semitones in `.scratch/chords-and-pitch/spec.md`, and all 8 Voicing names decode.
- [x] Unknown and half-blank names are refused.
- [x] Copy, Read, Track, Write and Push carry both types unchanged, and partial input is still refused.
- [x] Names that also spell a Number or Note (`69`, `13`, `D2`) are read by the operand position that receives them.
- [x] The Language Map and Source Paint have an operand type and colour for each, and the contrast checks cover them.
- [x] The nesting property tests generate both types.
- [x] `CONTEXT.md` defines Interval Set and Voicing, and Atom lists them.
