## Console palette

The egui console uses the prototype-inspired palette declared in `style.rs`:

- Page: `#101417` (`rgb(16, 20, 23)`)
- Source: `#0B1013` (`rgb(11, 16, 19)`)
- Cell grid line: `#1E292F` (`rgb(30, 41, 47)`)
- Ordinary Glyph: `#B9C5CA` (`rgb(185, 197, 202)`)
- Function: `#63D5B3` (`rgb(99, 213, 179)`)
- Bang and error: `#FF8585` (`rgb(255, 133, 133)`)
- Number: `#8FA7D8` (`rgb(143, 167, 216)`)
- Note: `#AE9FCD` (`rgb(174, 159, 205)`)
- Marker and Highlight: `#344149` (`rgb(52, 65, 73)`)
- Selection fill: `#11302A` (`rgb(17, 48, 42)`)
- Selection and Cursor stroke: `#63D5B3` (`rgb(99, 213, 179)`)

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
