# The sound module shows the synth's channels and effects as Cells

Status: accepted. Settled with Toby on 2026-10-27 in `.scratch/orcvs-site/` (ticket 15), after a
prototype of three looks. It amends [ADR 0087](0087-the-site-plays-the-console-through-a-built-in-synth.md):
Control Change 1 to 8 now set the Built-in Synth's effects.

ADR 0087 left the sound module empty: the Built-in Synth had no level or effect control, and the
effects panel was to come later and decide whether Control Change reaches it. Learn Orca's Enfer
shows the reference: eight global effects on CC 1 to 8 beside the console.

## Decision

**The sound module is a Grid of Cells in the row below the console on `/`.** It is drawn the way
the site draws a Source (`site/src/console/grid.css`): Cells 2 wide by 3 tall, in the console's
font and the Grayscale Theme colours. It reads as part of the console rather than a control panel
beside it. It holds two things, sixteen channel lights and eight effects.

**A channel light shows that a channel sounded.** Each of the sixteen channels has two Cells, the
first holding its hex digit. A Note On fills them with the magnitude colour at its velocity's
fraction of `7F`, and they clear 150 ms later.

**Each effect is a name, a value and a magnitude, two Cells each, then two spacer Cells.** A
blank row separates the channel lights from the effects.

- The name is two Cells in the Function role's colours.
- The value is the effect's `00`–`7F` as two hex digits in the Number role's colours.
- The magnitude is two Cells with a solid border in the magnitude colour, filled with that colour
  at the value's fraction of `7F`. It has no glyphs.

The effects run in CC order, four to a row:

| CC | Name | Effect | At load |
|---|---|---|---|
| 1 | `MD` | modulation: vibrato on the whole mix | `00` |
| 2 | `OF` | offset: each new sound starts this far into its course, which softens its attack | `00` |
| 3 | `LN` | length: scales how long each sound decays, and a tone's release | `7F` |
| 4 | `DL` | delay: an echo of the whole mix | `00` |
| 5 | `CH` | chorus on the whole mix | `00` |
| 6 | `EQ` | one tilt: `40` is flat, lower is darker and higher is brighter | `40` |
| 7 | `R1` | the smaller of two reverbs | `00` |
| 8 | `R2` | the larger reverb | `00` |

At the load values, the synth sounds exactly as it does with no effects. The synth has no samples,
so `OF` and `LN` act on its synthesised sounds rather than on a sample's start and length, as they
do in Enfer.

**Control Change 1 to 8 set the effects on any channel.** The effects are global, so a channel
picks nothing. A Source sets them with `!c`. The synth applies each change at its message's time,
like any other message. ADR 0087's other rules stand: All Sound Off and All Notes Off still release
a channel's tones, and every other Control Change is still ignored.

**The reader changes a value by dragging up or down on its effect, or by scrolling.** The sound
module sends each change to the synth as a Control Change message on its own page
(`{ type: "orcvs.midi", bytes, time }`), the message a console posts. The synth hears one stream,
and the sound module shows each value from that same stream, whoever sent it.

**Values reset on every load.** Nothing persists across page loads. A Source that needs an effect
sets it.

**The magnitude colour is a Theme colour, `sound.magnitude`.** Each built-in Theme sets it to its
`source.bang` colour, so it can differ from Bang without changing Bang. The site carries it as
`--orcvs-sound-magnitude`. The console's Theme format gains the key when the console draws the
sound module (ticket 14), since nothing in the console reads it until then.

**The effects land one at a time.** The sound module ships with all eight effects drawn and
taking values, plus `MD`'s sound. Each other effect's sound is its own ticket. Until it lands, an
effect shows and keeps its value but changes nothing.

## Rejected alternatives

**Page HTML controls: knobs, labels and lights styled in the Theme.** They read as a separate
control panel beside the console, which the site is not.

**Three-Cell names in a role of their own.** They set the names apart from Functions. Two Cells in
the Function colours read as part of the console, and the value and magnitude already show that
an effect is not Source.

**A bar of glyphs, such as `▮▮··`, for the magnitude.** Filling the Cells shows the level without
glyphs to read.

**Effects per channel, or Control Change on one channel only.** The synth has one mix, and ADR
0087 keeps velocity as a channel's only level.

**Keeping the reader's changes across loads.** A Source that sounds different on a second visit
cannot be shared.

## Consequences

- The synth core gains eight effect values, applied at frame time. `MD`, `DL`, `CH`, `EQ`, `R1` and
  `R2` process the whole mix, while `OF` and `LN` shape each new sound.
- The console has no new code: the sound module and the synth meet on the page, through messages
  the page already carries.
- The sound module is on `/` only. A page with the small console (ADR 0086) still plays the
  synth's effects as a Source sets them, but shows no sound module.
- Native draws the same sound module when it gets the synth (ticket 14).
