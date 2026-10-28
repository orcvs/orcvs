# 15 — Show the Sound Module, with modulation

**What to build:** The Sound Module as ADR 0088 decides it, in the row below the console on `/`:
sixteen channels and the Built-in Synth's eight effects, drawn as Cells in the console's font
and the Grayscale Theme colours. A Note On shows its channel as sounding. Control
Change 1 to 8 on any channel sets an effect, whether a Source sends it or the reader types a
value at the Cursor, and the Sound Module shows the value. Every value resets on load (`LN`
`7F`, `EQ` `40`, the rest `00`). The synth core plays `MD`, vibrato on the whole mix. The other
effects' sounds are tickets 17 to 23.

**Blocked by:** 08

**Status:** resolved

- [x] Synth core tests: at the load values the synth sounds as it does with no effects; CC 1 on any
      channel sets `MD`, and at `7F` a steady tone's pitch wavers while at `00` it does not; CC 2 to
      8 change no sound yet.
- [x] Site tests: a Note On fills its channel's light; a CC from the console shows as the effect's
      value and magnitude; typing a value at the Cursor sends its CC to the synth.
- [x] The magnitude colour is `--orcvs-sound-magnitude`, defaulting to each Theme's Bang colour.
- [x] Screenshots in the light and dark Themes and at 390 px wide.

## Comments

- Toby chose the look from `prototype/sound-module` (variant D), settled in three grilling rounds on
  2026-10-27: two-Cell names in the Function colours, the value in the Number colours, and a
  bordered magnitude filled at the value's fraction rather than a bar of glyphs.
- The Sound Module is `site/src/console/sound-module.ts` and `sound-module.css`, on `/` only. It
  listens to every `orcvs.midi` on the page and sends the reader's changes as `orcvs.midi` to its
  own window, which `synth-host.js` already plays, so the console and the host gain no code.
- `MD` is `synth/src/vibrato.rs`: a delay line whose delay swings from zero at 5 Hz, up to a
  semitone either way at `7F`. At `00` it reads the sample it was just given, so the mix is
  unchanged.
- The site's console comes from `main`'s `console-web` release, so `MD`'s sound reaches the site
  once this merges and the console is published again. The Sound Module itself ships with the site.
