# 15 — Show the Sound Module, with modulation

**What to build:** The Sound Module as ADR 0088 decides it, in the row below the console on `/`:
sixteen channel lights and the Built-in Synth's eight effects, drawn as Cells in the console's font
and the Grayscale Theme colours. A Note On fills its channel's light at its velocity. Control
Change 1 to 8 on any channel sets an effect, whether a Source sends it or the reader drags or
scrolls on the effect, and the Sound Module shows the value. The synth core keeps the eight
values, resets them on load (`LN` `7F`, `EQ` `40`, the rest `00`), and plays `MD`, vibrato on the
whole mix. The other effects' sounds are tickets 17 to 23.

**Blocked by:** 08

**Status:** ready-for-agent

- [ ] Synth core tests: at the load values the synth sounds as it does with no effects; CC 1 on any
      channel sets `MD`, and at `7F` a steady tone's pitch wavers while at `00` it does not; CC 2 to
      8 change no sound yet.
- [ ] Site tests: a Note On fills its channel's light; a CC from the console shows as the effect's
      value and magnitude; dragging on an effect sends its CC to the synth.
- [ ] The magnitude colour is `--orcvs-sound-magnitude`, defaulting to each Theme's Bang colour.
- [ ] Screenshots in the light and dark Themes and at 390 px wide.

## Comments

- Toby chose the look from `prototype/sound-module` (variant D), settled in three grilling rounds on
  2026-10-27: two-Cell names in the Function colours, the value in the Number colours, and a
  bordered magnitude filled at the value's fraction rather than a bar of glyphs.
