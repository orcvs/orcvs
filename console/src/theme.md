# Console palette

This is the decided console palette. `restyle-egui-console/03`
checks a capture against these tokens. A later palette change is a documented
change, not drift.

Under ADR 0053 every value on this page belongs to the Okabe–Ito built-in
Theme, the dark Theme the console ships with. Each one maps to a named property,
recorded in the Themes section below. Since `.scratch/theming/issues/06`, the
Source Grid, its borders and the Cursor Effect's colours paint from the
resolved Theme (`console/src/theme.rs`). Since `03`, the rest of the chrome —
`egui::Visuals`, including inherited toolkit colours such as weak text,
hyperlinks, code spans and the input caret, plus every chrome border and its
width — reads the same resolved Theme too (`console/src/style.rs::style`), and
the opaque window backdrop is a Theme property the resolver refuses to leave
nonopaque. Nothing the console draws is a compiled constant.

- Page: `#0B1112` (`rgb(11, 17, 18)`)
- Cell grid line: `rgba(29, 55, 49, 0.28)`
- 8 × 8 sector seam: `rgba(55, 101, 86, 0.43)`
- Cursor frame: `#EAEBE5` (`rgb(234, 235, 229)`)
- Cursor area: `#4CBE9C` (`rgb(76, 190, 156)`) at subdued, varying opacity
- Selection fill: `#0A2A22` (`rgb(10, 42, 34)`)
- Region fill: white at 17% opacity (`rgba(255, 255, 255, 0.17)`), a Theme value (`region.background`, ADR 0053)
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

The Cursor Effect's colours are Theme properties (`cursor.border`,
`cursor.area`, `cursor.background`, `region.background`,
`region.cursor.background`, `region.border`) with no control of their own,
and no "Reset" remains. Glitch amount and Glitch frequency are not Theme
values: ADR 0053 keeps them console settings, in the console's `Settings`
menu rather than any `Theme` menu (`.scratch/theming/issues/09`).
Amount zero retains one clear frame without decorative noise. Frequency zero
freezes both layers and stops their scheduled repaints. With persistence
enabled, both are restored independently of the saved Source, falling back
to their defaults — 60 and 55 — on an absent or malformed stored value.

ADR 0053 and `theming/09` retain those distinct appearances and also stop
cursor-effect repaints at amount zero, correcting the current frequency-only
scheduling rule. Frequency zero may retain decorative fragmentation; it does
not clear the effect. Reduced motion gives a clear, stationary frame without
changing the stored preferences. Playback, Run Clock and input-driven repaints
remain independent.

## Themes

ADR 0053 decides the model. One Theme styles the whole console: the Source Grid
and the chrome around it. Settings name a dark Theme, a light Theme, and a mode
(follow the operating system's appearance, or hold one of the two), and hold no
Theme values. A Theme uses one versioned Orcvs format: `format: orcvs-theme`, `version: 1`,
`name`, `inherits` and a `style` map of named properties. Built-ins define complete
values; custom files inherit omitted values. Base16 import is deferred.
Planned Theme controls are colours, opacity, Grid and Cell
colours, borders and border widths. Grid background and Cell fills expose colour and opacity. Cell grid lines and Sector Seams each expose independent colour, opacity and width. Cursor, Region and Diagnostic borders expose colour, opacity and width. Widths are bounded; transparent colours can hide lines. Grid line and border widths are measured in display points and retain the same visible thickness as Grid zoom changes; they do not scale with Cell size. Existing square Cells, zoom and spacing remain unchanged.
Exact key spellings and defaults are in
[the Theme specification](../../.scratch/theming/schema.md). The table below
records their relation to the current console. Preserve Grid structure,
existing fonts and layout, input behaviour and separate motion settings.
Font choice, font sizes, spacing, corner radii, shadows and gradients are not
planned. Leave room for future extensions without speculative machinery.
Grid/Cell border and Sector Seam widths accept finite values from 0 to 1 display point inclusive; chrome border widths accept 0 to 2 points inclusive. Width zero hides the stroke. Preserve normal-zoom defaults: Cell grid lines 0.5 points, Sector Seams 0.75 points, stationary Cursor/Region effect outlines 1 point, and existing visible chrome borders 1 point (absent borders remain absent). Fixed display-point widths intentionally replace the previous zoom-scaled stroke behaviour.

Cursor/Region effect width is nominal: retain the existing animated fragment variation of 0.45–1.25 times that width, without Grid zoom scaling. The 1-point maximum constrains nominal width, so an animated fragment can reach 1.25 points. Width zero hides every affected stroke; it does not implicitly disable separately coloured fills or change motion preferences.

Transparency reveals the underlying console surface; the application window remains opaque. Desktop/window transparency is outside this effort. Painting and contrast validation must use the same composited backgrounds.

A custom Theme inherits its parent's resolved named properties and replaces only properties explicitly supplied by the document. Omitted properties remain unchanged; no palette-slot recalculation exists.

The window backdrop and Grid background have separate Theme colours. The window backdrop must be opaque; panel, Grid and Cell layers above it may use alpha. A transparent Grid reveals the underlying console surface. Built-ins explicitly define both backgrounds; custom documents inherit them independently. Contrast validation uses the actual composite down to the opaque backdrop.

`cell.background` is a single Theme-wide base fill for all Cells, transparent by default. It composites over the Grid background and beneath Token tints, Diagnostic/Output Portal channels and Cursor/Region fills. It does not count as a fact fill when evaluating Region fallback, so setting it cannot suppress Region highlighting. Existing precedence among those highlighting channels remains unchanged. Individual Cells do not store styling.

Optional Cursor fills distinguish omission, none and explicit transparent colour. Omission inherits the parent value. None removes the optional fill and uses the existing fallback: for the Cursor inside a Region, it falls back to the ordinary Cursor fill; for the ordinary Cursor, it supplies no Cursor fill. An explicit transparent colour remains a supplied value and does not trigger that fallback. These optional states apply only to `cursor.background` and `region.cursor.background`, not to every colour key.

Role backgrounds are explicit colour properties, such as `source.function.background`, `source.number.background`, `diagnostic.background` and `output_portal.background`. They composite over the uniform `cell.background` under the confirmed fixed precedence. Changing a role foreground does not recalculate its background. There is no `fill_tint` property or shared tint-strength setting in the Orcvs Theme format. Extract and verify the built-in role background colours from current rendering so its appearance is preserved; illustrative colours in the interview are not accepted defaults.

There are no overrides: a different
look is a custom Theme that inherits from one built-in Theme and lists what it
changes. A custom Theme inherits its parent's dark/light appearance; a conflicting
explicit declaration is rejected. Custom Themes are authored and edited as files outside Orcvs; the console
loads and selects them and provides no Theme editor. Native Theme files live in
`~/.orcvs/themes/`, the only directory scanned at startup. Those files are
authoritative, so changes take effect only on the next launch without
re-importing; this release has no file watcher or reload action. Settings refer
to each native Theme by its filename stem; its declared name is a display label.
Built-in identities are reserved and cannot be replaced by files.
If multiple native files have the same filename stem, all files with that identity are refused and the conflict is reported with the conflicting filenames. Directory enumeration order never chooses a winner. If the selected identity is conflicted, use the default built-in Theme of the same appearance while retaining the saved selection; resolving the conflict restores the intended Theme on the next launch.
Web imports use files and the same filename-stem identity; a successful reimport
of the same identity updates its document. Imported documents remain in browser
storage when persistence is enabled. A missing or malformed selected native Theme file shows an error and
falls back to the default built-in Theme of the same appearance. The saved
selection is retained, so fixing the file restores it on the next launch.
Unknown appearance keys and out-of-range widths reject the entire Theme document. The error identifies the offending setting and explains the valid key or range; values are neither silently ignored nor clamped. A failed web reimport preserves the previous valid document. Contrast warnings alone do not reject a document.
Glitch amount and Glitch frequency are settings, not Theme values.

Themes choose colours and channels within fixed painting precedence. They cannot
reorder facts. The Function and Bang exceptions inside Output Portals, portal
precedence over other claims, Cursor fill precedence and Region fill fallback
described below remain in force under ADR 0053. Issue `06` carries the explicit
composition table and overlap tests for transparent, partial-alpha and opaque
channels.

The Okabe–Ito built-in Theme sets every named property explicitly, so that it
reproduces the values on this page:

| Property | Okabe–Ito value | Replaced control |
|---|---|---|
| `grid.background` Source background | `#000000` | Source colours → Source background |
| `source.comment` Comment | `#999999` | Source colours → Comment |
| `source.ordinary` Ordinary, Char, Atom | `#EAEBE5` | Source colours → Ordinary |
| `source.number` Number | `#56B4E9` | Source colours → Number |
| `source.note` Note | `#F0E442` | Source colours → Note |
| `source.function` Function | `#009E73` | Source colours → Function |
| `source.bang` Bang | `#CC79A7` | Source colours → Bang |
| `source.sequence` Sequence | `#0072B2` | Source colours → Sequence |
| `source.ordinary.background` | `#00000000` | No current role fill |
| `source.comment.background`, `source.bang.background` | `#00000000` | No current role fill |
| `source.function.background` | `#009E731A` | Function's own colour at 10% opacity |
| `source.number.background` | `#56B4E91A` | Number's own colour at 10% opacity |
| `source.note.background` | `#F0E4421A` | Note's own colour at 10% opacity |
| `source.atom.background` | `#EAEBE51A` | Atom has a fill despite sharing Ordinary foreground; Ordinary's own colour at 10% opacity |
| `source.sequence.background` | `#0072B21A` | Sequence's own colour at 10% opacity |
| `diagnostic.foreground` | `#D55E00` | Source colours → Diagnostic |
| `output_portal.foreground` | `#E69F00` | Source colours → Output Portal |
| `output_portal.background` | `#E69F001A` | Output Portal's own colour at 10% opacity |
| `grid.border` | `rgba(29, 55, 49, 0.28)` | fixed: Cell grid line |
| `sector.seam` | `rgba(55, 101, 86, 0.43)` | fixed: 8 × 8 sector seam |
| `cursor.border` | `#EAEBE5` | Cursor effects → Cursor colour |
| `cursor.area` | `#4CBE9C` | Cursor effects → Area colour |
| `cursor.background` | none | Cursor effects → Cursor cell colour |
| `region.background` | white at 17% | Cursor effects → Region colour |
| `region.cursor.background` | none | Cursor effects → Cursor colour in a Region |
| `window.background` | `#0B1112` | fixed: Page (the opaque window backdrop; `03`'s resolver refuses any other alpha) |
| `panel.background` | `#0B1112` | fixed: Page |
| `panel.border` | `#1C3932` (actual opaque conversion) | fixed: chrome stroke |
| `widget.inactive.border` | `#1C3932` | fixed: chrome stroke (width 0 keeps idle widgets' border absent) |
| `text` | `#EAEBE5` | fixed: widget text |
| `text.active` | `#65E6BE` | fixed: Selection and Cursor stroke (hovered/active widget text) |
| `text.muted` | `#E9EBE499` | fixed: egui's own weak-text attenuation (0.6 of `text`) |
| `input.background` | `#000000` | fixed: borrowed from Source background |
| `link` | `#5AAAFF` | fixed: egui's own default hyperlink colour |
| `code.background` | `#404040` | fixed: egui's own default code-span background |
| `input.cursor` | `#C0DEFF` | fixed: egui's own default text cursor and active IME underline |
| `selection.background` | `#0A2A22` | fixed: Selection fill |
| `selection.border` | `#65E6BE` | fixed: Selection and Cursor stroke |
| `selection.border.rest` | `#52C3A3` | fixed: Selection stroke while caret is hidden |
| `error`, `warning` | `#CC79A7` | fixed: borrowed from Bang |

`text.muted`, `link`, `code.background` and `input.cursor` are independent
Theme properties, not egui's own inherited defaults left unread: their
Okabe–Ito values equal what egui already defaulted to (0.6-attenuated `text`
for weak text, its own dark-mode hyperlink/code/cursor colours), which is why
today's appearance is unchanged, but a custom Theme now reaches all four
instead of the toolkit silently keeping its own second palette. The inactive
IME underline keeps egui's existing half-linear attenuation of `input.cursor`
rather than a separate property. `error` and `warning` are independent Theme
properties that preserve Bang's built-in value without being computed from it
— `.scratch/theming/issues/05`'s live borrow is gone.

The border widths are Theme properties too, in display points: `grid.border.width`
0.5, `sector.seam.width` 0.75, `cell.selection.border.width` 0.5,
`cursor.border.width` and `region.border.width` 1,
`diagnostic.border.width` and `output_portal.border.width` 0.5,
`panel.border.width`, `selection.border.width` and `widget.border.width` 1,
`widget.inactive.border.width` 0 (which is why that border stays absent at
its default) and `input.cursor.width` 2. `schema.md`
lists them with the chrome widths.

### Shipped Themes

| Identity | Appearance | Status |
|---|---|---|
| `okabe-ito` | dark | Built in; sets every named property at the values above. |
| `orcvs-light` | light | Reserved; its palette waits on `04`'s review. |

Settings save a dark and a light Theme identity, both `okabe-ito`, and restore
them unchanged. Nothing selects a Theme yet: `04` prepares and exposes the
dark and light pickers together, once the light Theme is accepted, so until
then the console always resolves `okabe-ito`.

## Source colours

Each Source Paint role paints its foreground and background from the resolved
Theme: `source.ordinary` (also Char and Atom), `source.comment`,
`source.function`, `source.bang`, `source.number`, `source.note`,
`source.sequence`, `diagnostic.*` and `output_portal.*`, each with its own
`.background`. `Theme → Source colours`, its Fill tint slider and its "Reset to
theme defaults" are gone, and the old `source_paint` storage key is left
unread.

The former Fill tint — each Token colour mixed 16% toward the Source
background — no longer sets the Okabe–Ito role backgrounds in the table
above: a 2026-09-22 retune replaced the extracted-opaque values with a
uniform 10% opacity instead, each role's own foreground colour stored
straight with alpha `0x1A` (`source.function.background`'s `#009E731A`, for
one) rather than a precomputed opaque mix over black. There is no
`fill_tint` property: changing a role's foreground does not recalculate its
background, and the 10% figure is baked into each stored value rather than a
shared scalar applied at paint time. Every recognized Function Cell — nested
Functions included — and
every Operand Cell (its declared Token: Number, Note, Atom, or Sequence,
whether the operand is still Pending, Valid, or Invalid) paints its role
background. Comment, Bang, an empty unclaimed Cell, and a Leftover Char have a
transparent one. A refused Function spelling takes none either (see
Diagnostic, below) — only a Function entry the Parser recognized does. Role
backgrounds composite over the uniform `cell.background`, which is transparent
in Okabe–Ito. On the Cursor's own Cell, the Cursor's fill wins outright.
Adjacent Cells that share one background paint as one run, the same
coalescing `Paint::background_runs` already gives the Cursor's and
Selection's fills.

ADR 0052 has each Render Cell carry the parser's shared Claim, which answers
whether its slot is written as it is built for the frame.
`RenderCell::source_paint` combines the Claim's Token, atom and written answer
into the Source Paint fact: Function, Pending Operand, Valid Operand, Invalid
Operand, Bang, Comment, or Unclaimed. Every Operand fact independently carries
the Token its Function signature declared. The console reads that fact and
never interprets the Claim's Span. Text that spells no Function — `hi`, both
Cells of a written `07`, a lone `|`, the trailing `<` of `<<<` — is Unclaimed
paint: the Parser's attempted Function classification is not a declared
expectation and therefore is not a Paint distinction.

An operand is Invalid when any Cell of its slot holds written content but the
slot did not bind, and Pending when the whole slot is blank. A Pending Cell answers its
declared Token colour rather than Diagnostic, and an Invalid one answers
Diagnostic on every Cell of its slot, the written ones and the blank ones
alike — `.+0`'s second operand, one Cell written and one blank, is Invalid
as a whole, so both of its Cells agree. Neither distinction is visible today:
`paint.rs`'s blank-glyph fallback leaves a Pending or Invalid Cell with no
content blank regardless of its foreground colour, so only the role
background shows on it, unchanged from `syntax-highlighting/03`. Evaluation-time operand
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
draws in the Output Portal colour on `output_portal.background`
instead of whatever its Source Paint fact alone would answer; an empty Output
Portal Cell shows the same background with no glyph. The one named exception is a
Bang answer: it keeps its own Bang glyph colour, because a Bang is what a
Producer emits rather than a value it writes, but still takes the Output
Portal's background in place of Bang's usual transparent one.

**Precedence where an Output Portal covers another Expression's claimed
Cells** — a consumer's operand, or another root, per `05`'s Overlap rule that
the fact "covers every Cell of the Reservation whatever else claims
it": a Cell that is itself a bound Function's own two-Cell spelling keeps its
Function paint outright, root or nested alike, because every Function's own
spelling already carries `Token::Function` regardless of nesting and telling
a root's spelling from a nested one would need the Expression this decision
does not read. Every other overlapping Cell — another root's own Number,
Note, Atom or Sequence operand among them — takes the Output Portal colour
and background instead of its own declared role, because the root's answer
is what a viewer reads there. The Cursor's own fill still wins outright over
everything above, on its own Cell.

The defaults, and the Okabe–Ito Theme's values for the same named properties,
are the Okabe–Ito colour-blind-safe assignment, as published in R
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

`console/src/contrast.rs::validate` (`.scratch/theming/issues/08`) measures
every reachable painted text state's *effective*, actually-composited
foreground against its actually-composited background — reusing
`style::cell_visuals_with_cursor_colour` and `style::cell_background`, the
same functions painting itself calls, so the two cannot independently drift.
Each result reports its role, its state (a `CursorPlacement` of `plain`,
`Cursor`, `Region` or `Region, Cursor's Cell`, crossed with whether the Cell
also lies in an Output Portal Reservation), the effective foreground and
background, the measured ratio against the 4.5:1 floor, and whether it is an
accepted exception. The floor is stated once, at `contrast::CONTRAST_FLOOR`,
from WCAG 2.1 Success Criterion 1.4.3 ("Contrast (Minimum)"); the returned
`ContrastReport` also carries the floor and a scope description directly, not
only in rustdoc. `validate` measures text contrast only: never Token-colour
distinguishability, colour-vision accessibility, border/focus visibility, or
the Cursor Effect's animated `area` field.

A Pending operand Cell draws no glyph, so `validate` has no Pending role to
measure — `Role` (Number, Note, Atom, Sequence) carries Valid and Invalid
only, and Atom/Sequence carry Invalid alone since neither ever binds.

Okabe–Ito's `plain`-placement measurements, after the user's 2026-09-22 retune
of every tinted role background to a uniform 10% opacity: Ordinary 17.51:1,
Bang 6.86:1 and Comment 7.37:1 (each against the bare `#000000` Source
background, since their own role backgrounds are transparent); Function
5.69:1, Number 8.19:1 and Note 13.64:1 (Valid), each against its own 10%
tint; `output_portal.foreground` 8.40:1 against an Unclaimed Cell's Output
Portal state, where the tint is `output_portal.background` alone (an
Unclaimed Cell's own background is transparent); `text` 15.88:1 against
`panel.background` and 17.51:1 against `input.background`; `text.muted`
(translucent, round-tripping to premultiplied bytes `[140, 141, 137, 153]`)
6.19:1 against `panel.background` and 6.29:1 against `input.background` —
none of these five changed, since the retune touched only the tinted role
backgrounds. Every invalid-operand Diagnostic state now clears the floor
too: Number 4.89:1, Note 4.66:1 and Atom 4.59:1, each `diagnostic.foreground`
(`#D55E00`) against that Token's own 10% tint; Sequence's own role,
`Sequence, Invalid`, measures 5.12:1. `contrast::tests::shipped_theme_gate`
runs as a real, non-`#[ignore]`d test: every reachable state clears 4.5:1,
so `accepted_failures` for `okabe-ito` is empty — there is nothing left to
except.

The single-Cell Cursor always replaces a role's background outright, whatever
that role's own background's own alpha — `style::cell_visuals_with_
cursor_colour`'s "the Cursor's own fill wins outright on its Cell." A Region
Cell, by contrast, only falls back to the Region fill when the role's own
background left nothing painted at all (alpha exactly `0`) — a role
background with any nonzero alpha, translucent or opaque, keeps winning
there. Both differ from `plain` only for a Theme whose Cursor/Region fills are
actually set; Okabe–Ito's are unset, so its own Cursor/Region states happen to
repeat `plain`'s figures without exercising either rule.

Sector boundaries are partial 0.75-pixel phosphor registration marks drawn over
Cell edges. Each sector corner forms a `+`: four equally strong arms fade toward
the midpoint between neighbouring corners with relative strengths `100, 72, 34,
13, 13, 34, 72, 100`. The faint middle also has sparse gaps derived from each
absolute Grid Position, so the marks feel imperfect without flicker. They
replace the historical `+` Marker Glyphs, leaving every empty Cell visually
empty while preserving the configured Sector Seam spacing as geometry.
