# Console Panel — parked design

**State:** parked mid-grilling. Twenty-one questions asked, none answered. Resume after
`source-grid-rendering` lands. Nothing here is decided; every arrow below is a recommendation
awaiting a yes or a no.

**What it is:** a telemetry surface for the console — tempo, Tick, playback state, Cursor context,
MIDI activity — that floats over the Source Grid, is dragged with the pointer, and settles onto a
fixed Anchor when released.

## Why it waits

Half of what it would display is unreachable. `PlaybackInner.tick` is private and
`PlaybackObservation` does not carry it; `Orcvs.playback_state` has no getter; the Cursor Position
has none either, and the tests scan `render_frame().rows()` for `cell.selected()` to find it
(`console.rs:739-749`). The Panel needs a public telemetry accessor on `Orcvs` before it can show a
real number, and that is a separate change to a separate crate.

It also draws over the Source Grid, which `source-grid-rendering` is rewriting. Designing an overlay
against a renderer being replaced is wasted work.

## Vocabulary proposed, not settled

**Panel** the floating surface, **Anchor** one of the fixed positions it settles onto, **Readout**
one labelled value inside it. `egui::Panel` is a real type in 0.36 and the console already calls
`egui::Panel::top` (`console.rs:425`), so the glossary entry needs an explicit
`_Avoid_: egui SidePanel, dock, HUD, toolbar`.

**BPM is missing from the glossary and present in the code** — `orcvs::opts::Bpm(NonZeroUsize)`,
`opts.rs:34-74`, default 20. `CONTEXT.md` has Tick Grid, tempo and Tick period and no BPM. Proposal
was to define BPM as the display form of the Tick Grid's period, so the domain term stays tempo.

**"Frame count" is one number, not two.** Orca's clock interval is `(60000 / bpm) / 4`
(`clock.js:140`); `Bpm::delay_ms` is `(60000 / bpm) / 4` (`opts.rs:69`). Identical. Orca's `f` is a
sixteenth-note step — what this glossary calls a **Tick** and explicitly tells you not to call a
frame. Render Frame rate is a separate thing and belongs in the Diagnostics window, not the Panel.

There is no runtime Turn counter. Commit `d7ac434` counts Turns in a test helper only.

## Recommendations on the record

Surface: screen space, not canvas space. Overlay, not space-reserving. One Panel prototyped,
architecture admitting several, stacking in Anchor order when two share one.

Anchors: twelve — `{TopLeft, TopRight, BottomLeft, BottomRight} x {Horizontal, Vertical}` plus
`{TopCentre, BottomCentre}` horizontal and `{LeftCentre, RightCentre}` vertical. Corner orientation
is an explicit toggle from a right-click menu, never inferred from drag direction, because inferring
makes the same drop land differently depending on approach path.

Magnetism: absolute. Every release settles to the nearest Anchor; there is no free-floating state.
That makes Panel position a twelve-value enum plus a transient drag position, rather than a `Pos2`
plus a maybe-Anchor — which matters more than it sounds, see the egui findings below.

Orientation changes shape *and* content: each Readout declares a long and a short form, Horizontal
uses long, Vertical short. A Panel that silently truncates is worse than one that abbreviates on
purpose.

Snap instantly, no settle animation. `style.rs:149` sets `animation_time: 0.0` beside
`CornerRadius::ZERO` and `Shadow::NONE`; the console has no animation anywhere by decision. If the
instant jump feels wrong, that is an argument to revisit the theme globally, not to except one
widget.

Container: bare `egui::Area` with every Readout non-interactive, not `egui::Window`. Tempo editing
already lives in the Tempo menu (`console.rs:492-505`); duplicating it into a draggable surface
creates a click-versus-drag ambiguity for no gain.

Keep the existing Diagnostics window (`console.rs:217-285`). It is developer-facing — FPS, frame
time, CPU, zoom, `inspection_ui`. The Panel is performer-facing. FPS belongs in Diagnostics because
a performer cannot act on it.

## egui findings, banked

**Nothing in the ecosystem does edge snapping.** Not `egui_dock` (tab docking), not `egui_tiles`
(tiling, no floating windows), not `egui_tool_windows` (floating, no magnetism). `egui_snap` does
not exist. Grepping egui itself for "snap" returns nothing, and there is no issue or discussion
proposing it. It is roughly forty lines of application code.

**`Area::anchor()` calls `movable(false)`.** Anchoring and dragging are mutually exclusive by
construction; you cannot re-anchor a dragged window. Compute the position yourself with
`Align2::align_size_within_rect(size, frame)` — the same call egui uses internally, so placement is
pixel-identical — and feed it through `current_pos()` each frame. The drag latch wins during a drag,
`current_pos` wins on idle frames, and the two never fight.

**`AreaState` cannot be written back.** `Areas::set_state` is `pub(crate)`; `Memory::area_rect`
reads and nothing writes. Position must be owned by the application. `AreaState::size` is
`#[serde(skip)]`, so a restored Area has no size for one frame — which an Anchor enum survives and
stored coordinates do not.

`Align2`'s nine constants are exactly four corners, four edge centres and the centre.

## Orca's HUD, for reference

Not a panel: a six-column by two-row tab grid drawn in the same cell coordinates as the program
(`client.js:321-339`), column width equal to the ruler spacing, fields hard-truncated at `grid.w - 1`
with no ellipsis — `Generator` renders as `Generat`.

Row 1: glyph or port name under the cursor, `x,y` with `+` in insert mode, selection `w:h`, `{f}f`
with `~` when paused, an IO activity meter, MIDI input device. Row 2: version then module count,
program `WxH`, ruler `w/h`, BPM, live variable keys, MIDI output device.

Its IO display is a level meter — one `|` per queued event summed across all backends, padded with
`.` — with no channels at all (`core/io.js:55-68`). Orca-c instead has a toggleable event list
printing `MIDI Note channel %d octave %d note %d velocity %d length %d` per event
(`tui_main.c:606-657`). Orcvs could do better than either: `OutputCommand` variants all carry
`MidiChannel` (`playback.rs:41-64`), so sixteen per-channel lights are available and neither Orca
has the information for them.

Worth taking from Orca: the beat flash (`*` every `f % 4 == 0`) and the glyph-under-cursor readout,
which is its most-used field. Worth leaving: the self-erasing device names and the marquee, both
workarounds for a seven-cell-wide field a real Panel does not have.

## Fog

Persistence of Anchor, orientation and visibility across sessions — `shell` has the `persistence`
feature on by default and this was never asked.

What happens to a Panel when the window is resized below the Panel's own size.

Whether the Panel is reachable by keyboard at all, and whether its Readouts should be in the tab
order.

Whether `!$`, the Application Command Function, should be able to toggle it.
