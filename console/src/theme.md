# Console palette

This is the decided console palette. `restyle-egui-console/03`
checks a capture against these tokens. A later palette change is a documented
change, not drift. The Source background and every Token's glyph colour used
to be pinned here too; `syntax-highlighting/01` moved them into their own
section below, as `Theme → Source colours` settings rather than fixed tokens.

- Page: `#0B1112` (`rgb(11, 17, 18)`)
- Cell grid line: `rgba(29, 55, 49, 0.28)`
- 8 × 8 sector seam: `rgba(55, 101, 86, 0.43)`
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

## Source colours

`Theme → Source colours` holds one opaque colour control per Source Paint
role: Source background, Ordinary (also Char and Atom), Comment, Function,
Bang, Number, Note, Sequence, Diagnostic, and Output Portal. The Cell grid line above
is not one of them and keeps its fixed colour regardless. Changes preview
immediately in the Source Grid; "Reset to theme defaults" restores every
Source colour together and leaves Cursor effects untouched. With persistence
enabled, Source colours are restored under their own key, independently of the
Source and of Cursor effects — an absent or malformed stored value falls back
to the defaults below rather than partly restoring.

The same section holds one more control that is not a colour: Fill tint, a
0-100% Slider defaulting to 16%. Every recognized Function Cell — nested
Functions included — and every Operand Cell (its declared Token: Number,
Note, Atom, or Sequence, whether the operand is still Pending, Valid, or
Invalid) paints a background tint of its Token colour mixed toward the Source
background by this percentage; 0% paints no tint at all. Comment, Bang, an
empty unclaimed Cell, and a Leftover Char are never tinted. A refused
Function spelling is not tinted either (see Diagnostic, below) — only a
Function entry the Parser recognized is. On the Cursor's own Cell, the
Cursor's fill wins over the tint outright. Adjacent tinted Cells that share
one colour paint as one run, the same coalescing `Paint::background_runs`
already gives the Cursor's and Selection's fills. "Reset to theme defaults"
restores Fill tint to 16% together with the ten colours, and persistence
restores it at the same key as the colours — there is no key of its own.

The console paints from the parser's own claim on a Cell (`orcvs::source::Claim
{ cells, token, atom }`, `RenderCell::claim()`) rather than from a Token and a
derived flag — `.scratch/syntax-highlighting/issues/09` folded the two
overlapping colour decisions that used to read those separately into the one
that reads the claim. `atom: None` marks a claim unbound, and what that
means depends on whether a signature declared the slot. An operand slot's
Token is its parent Function's declared expectation, so an unbound operand
claim whose content fails to bind is an Invalid Operand: it draws its glyph
in Diagnostic and keeps its declared Token's Fill tint (the declared Token is
still visible underneath, unchanged). A `Function` claim with no Atom is
different: the Parser seeds `Token::Function` at every Expression start as
the thing to try, not as anything declared, so text that spells no Function
there — `hi`, both Cells of a written `07`, a lone `|`, the trailing `<` of
`<<<` — expected nothing and failed nothing. It paints Ordinary with no
tint, the same as an unclaimed Cell. A Comment records no Atom too, but it
is a complete Language Unit rather than an invalid one (ADR 0035), so it is
never painted Diagnostic regardless of its own claim.

An unbound operand claim is Invalid when any Cell of its own slot
(`claim.cells`) holds written content, and Pending when the whole slot is
still entirely blank — a fact the claim cannot answer on its own, since
`atom: None` covers both alike (ADR 0044), so the console reads it from the
Render Frame's own Cell contents once per claim. A Pending Cell answers its
declared Token colour rather than Diagnostic, and an Invalid one answers
Diagnostic on every Cell of its slot, the written ones and the blank ones
alike — `.+0`'s second operand, one Cell written and one blank, is Invalid
as a whole, so both of its Cells agree. Neither distinction is visible today:
`paint.rs`'s blank-glyph fallback leaves a Pending or Invalid Cell with no
content blank regardless of its foreground colour, so only the Fill tint
shows on it, unchanged from `syntax-highlighting/03`. Evaluation-time operand
diagnostics are out of scope: they are Tick outcomes, not Source facts.

`syntax-highlighting/06` adds a Function's written value as a further input
to the same one decision: whether a Cell draws as a root Function's Output
Portal (`RenderCell::output_portal()`, `.scratch/syntax-
highlighting/issues/05`, `10` and `12`'s Answers) — the Cell pair one row
south of a scalar-only Function's anchor, or, for a Sequence-capable one, the
highlight fitted to its answer: at least four Cells from the Output Portal,
written or not, then the run of written Cells that follows, a Cell pair at a
time, stopping at the first blank Cell and never reaching past the root's
Reservation, which still runs to the end of that row. Four is the minimum
because a Function that never wrote more than two Cells would be declared
scalar, so it is what tells a Sequence-capable root from a scalar one on
sight. All of it is known from the current Source revision alone and so lit
before any Tick runs. Parsing is unchanged and unaware of it (`05`'s
Answer): a written scalar or Sequence answer re-parses exactly as ordinary
Source would (a `07` left south of `.+0304` is two unknown one-Cell
Functions, diagnostics included), and the Output Portal fact is what tells
that written value apart from the Expression that produced it. Such a Cell
draws in the Output Portal colour on the Output Portal's own Fill tint
instead of whatever that claim alone would answer; an empty Output
Portal Cell shows the same tint with no glyph. The one named exception is a
Bang answer: it keeps its own Bang glyph colour, because a Bang is what a
Producer emits rather than a value it writes, but still takes the Output
Portal's Fill tint in place of Bang's usual bare `None`.

**Precedence where an Output Portal covers another Expression's claimed
Cells** — a consumer's operand, or another root, per `05`'s Overlap rule that
the fact "covers every Cell of the Reservation whatever else claims
it": a Cell that is itself a bound Function's own two-Cell spelling keeps its
Function paint outright, root or nested alike, because every Function's own
spelling already carries `Token::Function` regardless of nesting and telling
a root's spelling from a nested one would need the Expression this decision
does not read. Every other overlapping Cell — another root's own Number,
Note, Atom or Sequence operand among them — takes the Output Portal colour
and tint instead of its own declared role, because the root's answer
is what a viewer reads there. The Cursor's own fill still wins outright over
everything above, on its own Cell.

The defaults are the Okabe–Ito colour-blind-safe assignment, as published in R
`grDevices`' `palette.colors("Okabe-Ito")` (Masataka Okabe & Kei Ito), chosen
in the Source Paint prototype
(`console/prototypes/syntax-highlighting/source-paint-prototype.html`,
`?variant=A&palette=okabe`). Output Portal — named Result before
`syntax-highlighting/06` renamed it to match `05`'s decided vocabulary — was
exposed as a setting before it had a painter; it has one now.

- Source background: `#000000` (`rgb(0, 0, 0)`) — Okabe–Ito black
- Ordinary, Char, and Atom: `#EAEBE5` (`rgb(234, 235, 229)`) — not a named Okabe–Ito swatch but the Cursor frame's off-white, so plain Source text and the frame drawn over it read as one white; a fixed default that copies that colour rather than following the Cursor setting (`syntax-highlighting/07`, which moved it from the prototype's `#FFFFFF`)
- Comment: `#999999` (`rgb(153, 153, 153)`) — Okabe–Ito gray
- Function: `#009E73` (`rgb(0, 158, 115)`) — Okabe–Ito bluish green
- Bang: `#CC79A7` (`rgb(204, 121, 167)`) — Okabe–Ito reddish purple
- Number: `#56B4E9` (`rgb(86, 180, 233)`) — Okabe–Ito sky blue
- Note: `#F0E442` (`rgb(240, 228, 66)`) — Okabe–Ito yellow
- Sequence: `#0072B2` (`rgb(0, 114, 178)`) — Okabe–Ito blue
- Diagnostic: `#D55E00` (`rgb(213, 94, 0)`) — Okabe–Ito vermillion, an unbound entry's glyph colour since `syntax-highlighting/04`
- Output Portal: `#E69F00` (`rgb(230, 159, 0)`) — Okabe–Ito orange, a Function's written value since `syntax-highlighting/06`

Every Source colour meets WCAG AA's 4.5:1 floor against the Source background
except Sequence: `#0072B2` measures 4.05:1 on `#000000`, the Okabe–Ito
assignment's own choice, kept as a named exception rather than silently
relaxing the floor. Comment, at 7.37:1, reads dimmer than Ordinary — the same
relationship the previous palette held — but is no longer the dimmest colour
above the floor: Diagnostic (5.43:1), Function (6.14:1) and Bang (6.86:1) all
read dimmer than Comment while still clearing 4.5:1. That is restated here
rather than left as an implied ordering the new defaults do not hold.

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
