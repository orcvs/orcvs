# Console palette

This is the decided console palette. `restyle-egui-console/03`
checks a capture against these tokens. A later palette change is a documented
change, not drift.

- Page: `#0B1112` (`rgb(11, 17, 18)`)
- Source: `#070D0D` (`rgb(7, 13, 13)`)
- Cell grid line: `rgba(29, 55, 49, 0.28)`
- 8 × 8 sector seam: `rgba(55, 101, 86, 0.43)`
- Ordinary Glyph: `#A5B7B2` (`rgb(165, 183, 178)`)
- Comment: `#7A8784` (`rgb(122, 135, 132)`) — the ordinary Glyph dimmed, at 5.25:1 against the Cell ground
- Function: `#68E0B8` (`rgb(104, 224, 184)`)
- Bang and error: `#FF7F87` (`rgb(255, 127, 135)`)
- Number: `#83A6D8` (`rgb(131, 166, 216)`)
- Note: `#AA91D6` (`rgb(170, 145, 214)`)
- Cursor frame: `#EAEBE5` (`rgb(234, 235, 229)`)
- Cursor area: `#4CBE9C` (`rgb(76, 190, 156)`) at subdued, varying opacity
- Selection fill: `#0A2A22` (`rgb(10, 42, 34)`)
- Region fill: white at 17% opacity (`rgba(255, 255, 255, 0.17)`), adjustable in `Theme → Cursor effects`
- Selection stroke while caret is hidden: `#52C3A3` (`rgb(82, 195, 163)`)
- Selection and Cursor stroke: `#65E6BE` (`rgb(101, 230, 190)`)

The Cursor is an eroded off-white frame whose four edges change independently.
Bright fragments, gaps, short horizontal tears, and fine connections evolve at
irregular intervals while the selected Cell and its Glyph remain exact. A
faint green field beneath the Grid extends roughly seven Cells around it. The
field uses continuous positions, related concentrations, large empty patches,
and horizontal interruption rather than colouring whole Cells. Its broad form
changes more slowly than its grain. The effect advances from console
presentation time, independently of Playback and Source revisions, and remains
stable between its scheduled visual changes.

A Region larger than one Cell is outlined by the same eroded frame, drawn
around the whole Region: the lasso. Each Cell-length of its edges carries the
Cursor frame's fragments at the Cursor frame's weight, and the Cursor's own
Cell border is hidden while the lasso stands. Every other Cell of the Region
takes the Region fill; the Cursor's Cell takes the Cursor colour in a Region,
which is unset by default and then leaves that Cell the Cursor cell colour, or
the Source ground when that is unset too. Inside the lasso the Cursor's Cell is
ruled as every other Cell of the Region, sector seams included.

`Theme → Cursor effects` holds the deliberate adjustments: Cursor colour,
Area colour, Region colour (with opacity), Cursor colour in a Region (with
opacity), Cursor cell colour, Glitch amount,
and Glitch frequency. Colour changes are explicit overrides of the defaults
above; reset restores every default together.
Amount zero retains one clear frame without decorative noise. Frequency zero
freezes both layers and stops their scheduled repaints. With persistence
enabled, these preferences are restored independently of the saved Source.

Sector boundaries are partial 0.75-pixel phosphor registration marks drawn over
Cell edges. Each sector corner forms a `+`: four equally strong arms fade toward
the midpoint between neighbouring corners with relative strengths `100, 72, 34,
13, 13, 34, 72, 100`. The faint middle also has sparse gaps derived from each
absolute Grid Position, so the marks feel imperfect without flicker. They
replace the historical `+` Marker Glyphs, leaving every empty Cell visually
empty while preserving the configured Sector Seam spacing as geometry.

The historical base16 palette below is retained as design context; it is not the
console's rendering source of truth.

base00: | #22273b | rgb(34, 39, 59)
base01: | #414f60 | rgb(65, 79, 96)
base02: | #5a8380 | rgb(90, 131, 128)
base03: | #6e6f72 | rgb(110, 111, 114)
base04: | #87888b | rgb(135, 136, 139)
base05: | #a4a6a9 | rgb(164, 166, 169)
base06: | #c7c9cd | rgb(199, 201, 205)
base07: | #8dbdaa | rgb(141, 189, 170)
base08: | #777abc | rgb(119, 122, 188)


base09: | #94929e | rgb(148, 146, 158)
base0A: | #4f9062 | rgb(79, 144, 98)
base0B: | #6562a8 | rgb(101, 98, 168)
base0C: | #226f68 | rgb(34, 111, 104)
base0D: | #4d6bb6 | rgb(77, 107, 182)
base0E: | #716cae | rgb(113, 108, 174)
base0F: | #8c70a7 | rgb(140, 112, 167)




base00 - Default Background
base01 - Lighter Background (Used for status bars, line number and folding marks)
base02 - Selection Background
base03 - Comments, Invisibles, Line Highlighting
base04 - Dark Foreground (Used for status bars)
base05 - Default Foreground, Caret, Delimiters, Operators
base06 - Light Foreground (Not often used)
base07 - Light Background (Not often used)
base08 - Variables, XML Tags, Markup Link Text, Markup Lists, Diff Deleted



base09 - Integers, Boolean, Constants, XML Attributes, Markup Link Url
base0A - Classes, Markup Bold, Search Text Background
base0B - Strings, Inherited Class, Markup Code, Diff Inserted
base0C - Support, Regular Expressions, Escape Characters, Markup Quotes
base0D - Functions, Methods, Attribute IDs, Headings
base0E - Keywords, Storage, Selector, Markup Italic, Diff Changed
base0F - Deprecated, Opening/Closing Embedded Language Tags, e.g. <?php ?>
