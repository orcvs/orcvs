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
beside it. It holds two things, sixteen channels and eight effects.

**A channel shows that it sounded.** The sixteen channels run across one row, each a single Cell
holding its hex digit with a spacer Cell after it. A channel is dim until a Note On sounds it, then
takes the Function role's colours, and goes dim again 150 ms later. A channel shows no level.

**Each effect is a column two Cells wide: its magnitude, then its value, then its name, a row
each.** Two spacer Cells separate one effect's column from the next, and a blank row separates the
channels from the effects.

- The magnitude is two Cells with a solid border in the magnitude colour, filled with that colour
  at the value's fraction of `7F`. It has no glyphs.
- The value is the effect's `00`–`7F` as two hex digits in the Number role's colours.
- The name is two Cells in the Function role's colours.

The effects run in CC order, eight across:

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

**The Sound Module takes one Theme colour, `sound.magnitude`.** It draws each magnitude's border
and fill. Each built-in Theme sets it to its `source.bang` colour, so it can differ from Bang
without changing Bang. The site carries it as `--orcvs-sound-magnitude`. The console's Theme format
gains the key when the console draws the Sound Module (ticket 14), since nothing in the console
reads it until then.

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

**Channel lights that fill with the magnitude colour at a Note On's velocity.** A filled channel
beside filled magnitudes reads as one of the effects, and a lit digit needs a second Theme colour to
stay legible on its fill. A channel stacked over an effect's column reads as that effect's channel.

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
