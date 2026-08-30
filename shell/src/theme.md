# Console palette

The egui console uses the prototype-inspired palette declared in `style.rs`:

- Page: `#0B1112` (`rgb(11, 17, 18)`)
- Source: `#070D0D` (`rgb(7, 13, 13)`)
- Cell grid line: `rgba(29, 55, 49, 0.28)`
- 8 × 8 sector seam: `rgba(55, 101, 86, 0.43)`
- Ordinary Glyph: `#A5B7B2` (`rgb(165, 183, 178)`)
- Function: `#68E0B8` (`rgb(104, 224, 184)`)
- Bang and error: `#FF7F87` (`rgb(255, 127, 135)`)
- Number: `#83A6D8` (`rgb(131, 166, 216)`)
- Note: `#AA91D6` (`rgb(170, 145, 214)`)
- Marker: `rgba(46, 82, 72, 0.44)`
- Highlight: `#2A5A4E` (`rgb(42, 90, 78)`)
- Cursor focus core: fill `#0A1E1A`, line `rgba(76, 190, 156, 0.59)`
- Cursor focus inner: fill `#091A17`, line `rgba(58, 148, 122, 0.49)`
- Cursor focus middle: fill `#081614`, line `rgba(43, 110, 92, 0.39)`
- Cursor focus outer: fill `#081211`, line `rgba(34, 78, 67, 0.32)`
- Selection fill: `#0A2A22` (`rgb(10, 42, 34)`)
- Selection stroke while caret is hidden: `#52C3A3` (`rgb(82, 195, 163)`)
- Selection and Cursor stroke: `#65E6BE` (`rgb(101, 230, 190)`)

The Cursor field is a seven-Cell Cartesian focus matrix. Cell-aligned square
bands have widths `1 : 1 : 2 : 3`, giving cumulative radii of 1, 2, 4, and 7
Cells. Background changes stay near-black; most of the focus is expressed by
grid-line energy. This reads as an address reticle rather than radial light.
At each band boundary, a deterministic hash of the absolute Grid Position
pushes roughly half of Cells into the next band. The resulting chipped edges
interact with the fixed Source coordinates without random flicker. The outer
boundary uses denser breakup, dropping roughly two thirds of its edge Cells.
Because the hash belongs to the absolute Grid Position, each Cell's noise is
stable while Cursor movement samples a different boundary pattern.

Sector boundaries are partial 0.75-pixel phosphor registration marks drawn over
Cell edges. Each sector corner forms a `+`: four equally strong arms fade toward
the midpoint between neighbouring corners with relative strengths `100, 72, 34,
13, 13, 34, 72, 100`. The faint middle also has sparse gaps derived from each
absolute Grid Position, so the marks feel imperfect without flicker. They
replace the historical `+` Marker Glyphs, leaving every empty Cell visually
empty while preserving the configured Marker spacing as geometry.

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
