# Console palette

This is the decided console palette. A later palette change is a documented
change, not drift.

Under ADR 0053 the page records the two built-in Themes, Grayscale Dark and
Grayscale Light, each appearance's default. Each value maps to a named
property, recorded in the Themes section below. Since `.scratch/theming/issues/06`, the
Source Grid, its borders and the Cursor's and Region's colours paint from the
resolved Theme (`console/src/theme.rs`). Since `03`, the rest of the chrome —
`egui::Visuals`, including inherited toolkit colours such as weak text,
hyperlinks, code spans and the input caret, plus every chrome border and its
width — reads the same resolved Theme too (`console/src/style.rs::style`), and
the opaque window backdrop is a Theme property the resolver refuses to leave
nonopaque. Nothing the console draws is a compiled constant.

The Cursor is an outline on its Cell's four edges, in `cursor.border`, drawn once over
every line beside it and the same on all four sides (ADR 0078). It is still:
nothing about the Cursor moves or schedules a repaint of its own.

A Region larger than one Cell is outlined the same way around the whole
Region, its perimeter's edges in `region.border`: the lasso. The Cursor's own
Cell keeps only its own fact's border while the lasso stands. Every other Cell of the Region
takes the Region fill; the Cursor's Cell takes the Cursor colour in a Region,
which is unset by default and then leaves that Cell the Cursor cell colour, or
the Source ground when that is unset too. Inside the lasso the Cursor's Cell is
ruled as every other Cell of the Region, sector seams included.

The Cursor's and Region's colours are Theme properties (`cursor.border`,
`cursor.background`, `cursor.foreground`, `region.background`,
`region.cursor.background`, `region.cursor.foreground`, `region.border`) with
no control of their own, and no "Reset" remains.

## Themes

ADR 0053 decides the model. One Theme styles the whole console: the Source Grid
and the chrome around it. Settings name a dark Theme, a light Theme, and a mode
(follow the operating system's appearance, or hold one of the two), and hold no
Theme values. A Theme uses one versioned Orcvs format: `format: orcvs-theme`, `version: 1`,
`name`, `inherits` and a `style` map of named properties. Built-ins define complete
values; custom files inherit omitted values. Base16 import is deferred.
Planned Theme controls are colours, opacity, Grid and Cell
colours, borders and border widths. Grid background and Cell fills expose colour and opacity. Cell grid lines and Sector Seams each expose independent colour, opacity and width. Cursor, Region and Diagnostic borders expose colour, opacity and width. Widths are bounded; transparent colours can hide lines. Grid line and border widths are measured in display points and retain the same thickness in points at every egui zoom factor; they do not scale with Cell size. A Theme sets no Cell size or spacing.
Exact key spellings and ranges are in
[the Theme specification](../../.scratch/theming/schema.md), and each
built-in's table below records its values. Preserve Grid structure,
existing fonts and layout, and input behaviour.
Font choice, font sizes, spacing, corner radii, shadows and gradients are not
planned. Leave room for future extensions without speculative machinery.
Grid/Cell border and Sector Seam widths accept finite values from 0 to 1 display point inclusive; chrome border widths accept 0 to 2 points inclusive. Width zero hides the stroke; a Source Cell border of width zero withdraws from the edges it would draw, and the next fact's line, or the Grid line, shows there. Each Cell edge is drawn once, in the highest-ranked border either Cell carries on that side (ADR 0078). Preserve the existing defaults: Cell grid lines 0.5 points, Sector Seams 0.75 points, Cursor/Region outlines 1 point, and existing visible chrome borders 1 point (absent borders remain absent). Widths are fixed display points: a stroke never scales with the Cell size.

A Cursor/Region outline of width zero withdraws from its edges; it does not disable the separately coloured fills.

Transparency reveals the underlying console surface; the application window remains opaque. Desktop/window transparency is outside this effort. Painting and contrast validation must use the same composited backgrounds.

A custom Theme inherits its parent's resolved named properties and replaces only properties explicitly supplied by the document. Omitted properties remain unchanged; no palette-slot recalculation exists.

The window backdrop and Grid background have separate Theme colours. The window backdrop must be opaque; panel, Grid and Cell layers above it may use alpha. A transparent Grid reveals the underlying console surface. Built-ins explicitly define both backgrounds; custom documents inherit them independently. Contrast validation uses the actual composite down to the opaque backdrop.

`cell.background` is a single Theme-wide base fill for all Cells, transparent by default. It composites over the Grid background and beneath Token tints, Diagnostic/Output Portal channels and Cursor/Region fills. It does not count as a fact fill when evaluating Region fallback, so setting it cannot suppress Region highlighting. Existing precedence among those highlighting channels remains unchanged. Individual Cells do not store styling.

Optional Cursor colours distinguish omission, none and explicit transparent colour. Omission inherits the parent value. None removes the optional colour and uses its fallback: for the Cursor inside a Region, the fill falls back to the ordinary Cursor fill and the glyph colour to the ordinary Cursor glyph colour; for the ordinary Cursor, it supplies no Cursor fill; and a glyph with no Cursor glyph colour keeps its role colour. An explicit transparent colour remains a supplied value and does not trigger that fallback: a transparent `cursor.foreground` hides the glyph. These optional states apply only to `cursor.background`, `cursor.foreground`, `region.cursor.background` and `region.cursor.foreground`, not to every colour key.

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
The web console loads no Theme files and has the built-ins alone; web Theme
import is deferred past v1. A missing or malformed selected native Theme file shows an error and
falls back to the default built-in Theme of the same appearance. The saved
selection is retained, so fixing the file restores it on the next launch.
Unknown appearance keys and out-of-range widths reject the entire Theme document. The error identifies the offending setting and explains the valid key or range; values are neither silently ignored nor clamped. Contrast warnings alone do not reject a document.

Themes choose colours and channels within fixed painting precedence. They cannot
reorder facts. The Function and Bang exceptions inside Output Portals, portal
precedence over other claims, Cursor fill precedence and Region fill fallback
described below remain in force under ADR 0053. Issue `06` carries the explicit
composition table and overlap tests for transparent, partial-alpha and opaque
channels.

`text.muted`, `link`, `code.background` and `input.cursor` are independent
Theme properties, not egui's own inherited defaults left unread, so a custom
Theme reaches all four and the toolkit keeps no second palette of its own. The
inactive IME underline keeps egui's half-linear attenuation of `input.cursor`
rather than a separate property. `error` and `warning` are independent Theme
properties, not computed from Bang.

`code.background` is egui's `code_bg_color`: the fill behind text egui lays
out as code, `RichText::code` or `Ui::code`, which it sets in monospace over
that fill. It is not Orcvs Source, which the Grid paints from its own roles.
The console shows no such text, so the property paints nothing today; it is
mapped so that text marked as code takes the Theme's colour rather than
egui's own default.

The border widths are Theme properties too, in display points, and each
built-in's table below records its widths. `schema.md` lists them with the
chrome widths. `overview.source_view.width` is a chrome width, and
`widget.inactive.border.width` 0 keeps an idle widget's border absent.
`cell.selection.border.width` is accepted and read by nothing: the Cursor's
outline is its Cell's edges, at `cursor.border.width`.

A visible role border frames every Cell its role claims, whether the Cell holds content or not, so a Comment's blank Cells and a Pending operand slot take their role's border. An unclaimed Cell draws as Ordinary, and takes the Ordinary border only while it holds content, so an empty Cell keeps the Grid line at `grid.border.width`.

### Input Portal

Input Portal marks a Cell a Function reads. It is a Theme role with the same
channels as Output Portal (`input_portal.foreground`, `.background`, `.border`
and `.border.width`) and the same precedence: it paints over every fact but a
Function and a Bang, Diagnostic included; a bound Function and a Bang keep all
their own paint; the Cursor and Region keep their precedence over it. Where a Cell plays both roles, Output
Portal's channels paint it and Input Portal's do not, so a Cell a Function both
reads and writes shows the write.

### Locked

Locked marks the two Cells of the Function of an Expression root a Halt's lock
withheld in the last Tick, as Orca draws the Cells an operator locks in a style
of their own. It is a Theme role with the same channels as the Portals
(`locked.foreground`, `.background`, `.border` and `.border.width`), and paints
only a Cell that plays no Portal role: a locked Cell a Function also reads or
writes paints as that read or write, as Orca draws a port over a locked Cell,
so a locked Function's spelling there keeps its own paint and shows no lock.
A lock marks nothing but a Function's own spelling, so unlike the Portals
Locked paints a bound Function and a Bang too; the contrast validator measures
it only over those two facts, the only ones it can land on.

### Buttons

The Panel's Play and Rewind controls are Cells, `P` and `R`, drawn in the
`button` role: `button.foreground`, `.background`, `.border` and
`.border.width`. A pressed button, as `P` is while Playback plays, draws in
`button.pressed.foreground`, `.background` and `.border` at the same width.
The contrast validator holds both glyphs to the floor over their fills. Both
built-ins draw a button as a Function Cell, and a pressed one filled and
bordered in the Bang ink, under a black glyph in Grayscale Dark and a white
one in Grayscale Light.

### The grayscale built-in Themes

`grayscale-dark` and `grayscale-light` are each appearance's default and
fallback. Both tell Source roles apart by grey level and by filled and
bordered Number, Note, Function and Bang Cells, and keep hue for the two roles
that must stand out: Grayscale Dark draws Bang in green and the Diagnostic
glyph in red, and Grayscale Light draws Bang in magenta and frames a
Diagnostic in red. Both fill the Region with an opaque grey.

Where two glyph roles read alike, closer than the colour-vision floor on the
Grid background, the Cell around the glyph tells them apart
(`theme::tests::roles_whose_glyphs_read_alike_differ_in_fill_or_border`):

| Glyphs that read alike | Grayscale Dark tells them apart by | Grayscale Light tells them apart by |
|---|---|---|
| Ordinary, Note | Note's fill and border | Note's fill and border |
| Output Portal, Input Portal | Input Portal's border | nothing: both Portals draw alike |

The test accepts the two Portals alike in any built-in, since both mark a Cell
the Tick reached.

Both draw Comment faint, below the 4.5:1 floor, which Comment is not held to
(`contrast::Role::held_to_floor`): Grayscale Dark's `#555555` measures 2.82:1
on black and 1.57:1 on the Region fill, and Grayscale Light's `#ADADAD`
measures 2.24:1 on white and 1.72:1 on the Region fill. The Input Portal glyph in
Grayscale Light is `#E4E4E4`, which measures 4.52:1 on its `#666666` fill.

### The Grayscale Dark built-in Theme

`grayscale-dark` sets light grey text and Cells on black, never pure white.
The Output Portal frames its Cells unfilled in `#444444` under a `#E6E6E6`
glyph, and the Input Portal fills its Cells light grey under a dark
glyph. The Cursor draws every glyph in `#E6E6E6`.
`theme::tests::every_built_in_defines_every_key_at_its_table` reads the
table below and fails when it and the code disagree, or when it leaves a
property out.

| Property | Grayscale Dark |
|---|---|
| `cursor.background`, `region.cursor.foreground` | none |
| `cursor.foreground`, `source.function`, `output_portal.foreground`, `cursor.border`, `selection.border`, `text.active`, `link`, `input.cursor`, `error`, `warning`, `button.foreground` | `#E6E6E6` |
| `region.cursor.background`, `input_portal.foreground`, `selection.background` | `#333333` |
| `window.background`, `panel.background`, `grid.background`, `locked.background`, `input.background`, `button.pressed.foreground` | `#000000` |
| `cell.background`, `source.ordinary.background`, `source.comment.background`, `source.ordinary.border`, `source.comment.border`, `diagnostic.background`, `output_portal.background`, `locked.foreground` | `#00000000` |
| `source.ordinary`, `diagnostic.border`, `text` | `#C8C8C8` |
| `source.comment` | `#555555` |
| `source.number` | `#898989` |
| `source.note`, `input_portal.background`, `locked.border` | `#CCCCCC` |
| `source.bang`, `source.bang.border`, `button.pressed.background`, `button.pressed.border` | `#00FF00` |
| `source.number.background`, `source.note.background`, `source.number.border`, `source.note.border` | `#222222` |
| `source.function.background`, `source.function.border`, `output_portal.border`, `button.background`, `button.border` | `#444444` |
| `source.bang.background` | `#111111` |
| `diagnostic.foreground` | `#FF3B3B` |
| `input_portal.border` | `#FFFFFF66` |
| `grid.border` | `#FFFFFF1A` |
| `sector.seam` | `#FFFFFF33` |
| `region.border` | `#DDDDDD` |
| `region.background` | `#383838` |
| `panel.border`, `widget.inactive.border`, `code.background` | `#2A2A2A` |
| `panel.separator` | `#555555` |
| `selection.border.rest` | `#A8A8A8` |
| `text.muted` | `#C8C8C899` |
| `grid.border.width`, `input_portal.border.width`, `source.ordinary.border.width`, `source.comment.border.width`, `source.bang.border.width` | 0.5 |
| `sector.seam.width` | 0.75 |
| `cell.selection.border.width`, `cursor.border.width`, `region.border.width`, `output_portal.border.width`, `locked.border.width`, `source.number.border.width`, `source.note.border.width`, `source.function.border.width`, `panel.border.width`, `selection.border.width`, `widget.border.width`, `button.border.width` | 1 |
| `diagnostic.border.width`, `widget.inactive.border.width` | 0 |
| `input.cursor.width`, `panel.separator.width` | 2 |
| `overview.source_view` | `#FFFFFF` |
| `overview.source_view.width` | 1 |

### The Grayscale Light built-in Theme

`grayscale-light` sets dark grey text and Cells on white. The Output Portal
frames its Cells in `#BBBBBB` with no fill under a `#191919` glyph, the
Input Portal draws a light glyph on a `#666666` fill, and a Diagnostic draws its red glyph and
border on a white fill.
`theme::tests::every_built_in_defines_every_key_at_its_table` reads the
table below and fails when it and the code disagree, or when it leaves a
property out.

| Property | Grayscale Light |
|---|---|
| `cursor.background`, `region.cursor.foreground` | none |
| `cursor.foreground`, `source.function`, `output_portal.foreground`, `cursor.border`, `region.border`, `selection.border`, `text.active`, `link`, `input.cursor`, `error`, `warning`, `button.foreground` | `#191919` |
| `region.cursor.background`, `selection.background` | `#CCCCCC` |
| `window.background`, `panel.background`, `grid.background`, `source.bang.background`, `diagnostic.background`, `locked.background`, `input.background`, `button.pressed.foreground` | `#FFFFFF` |
| `cell.background`, `source.ordinary.background`, `source.comment.background`, `source.ordinary.border`, `source.comment.border`, `output_portal.background`, `locked.foreground` | `#00000000` |
| `source.ordinary`, `text` | `#373737` |
| `source.comment` | `#ADADAD` |
| `source.number` | `#616161` |
| `source.note`, `locked.border` | `#333333` |
| `source.bang`, `button.pressed.background`, `button.pressed.border` | `#D100D1` |
| `source.number.background`, `source.note.background`, `source.number.border`, `source.note.border` | `#DDDDDD` |
| `source.function.background`, `source.function.border`, `output_portal.border`, `button.background`, `button.border` | `#BBBBBB` |
| `source.bang.border` | `#FF00FF` |
| `diagnostic.foreground`, `diagnostic.border` | `#C70000` |
| `input_portal.foreground` | `#E4E4E4` |
| `input_portal.background`, `input_portal.border` | `#666666` |
| `grid.border` | `#0000001A` |
| `sector.seam` | `#00000033` |
| `region.background` | `#E1E1E1` |
| `panel.border`, `widget.inactive.border`, `code.background` | `#D5D5D5` |
| `panel.separator` | `#ADADAD` |
| `selection.border.rest` | `#575757` |
| `text.muted` | `#373737AF` |
| `grid.border.width`, `cell.selection.border.width`, `input_portal.border.width`, `source.ordinary.border.width`, `source.comment.border.width` | 0.5 |
| `sector.seam.width` | 0.75 |
| `cursor.border.width`, `region.border.width`, `diagnostic.border.width`, `output_portal.border.width`, `locked.border.width`, `source.number.border.width`, `source.note.border.width`, `source.function.border.width`, `source.bang.border.width`, `panel.border.width`, `selection.border.width`, `widget.border.width`, `button.border.width` | 1 |
| `widget.inactive.border.width` | 0 |
| `input.cursor.width`, `panel.separator.width` | 2 |
| `overview.source_view` | `#000000` |
| `overview.source_view.width` | 1 |

### Shipped Themes

| Identity | Appearance | Status |
|---|---|---|
| `grayscale-dark` | dark | Built in and the dark default; sets every named property at the values above. |
| `grayscale-light` | light | Built in and the light default; sets every named property at the values above. |

### Selecting a Theme

The mode — **Follow the OS**, **Dark** or **Light** — is three icon buttons
(◐ ☾ ☼) at the right of the top bar, each explained on hover. The dark and
light Themes are settings, not controls: on native, `~/.orcvs/config.toml`
names them, read once at startup; the web runs on the built-ins.

```toml
[theme]
dark = "grayscale-dark"    # the default
light = "grayscale-light"  # the default

[cell]
width = 12              # points, 8–64
height = 18             # points, 8–64
```

Every key is optional and an absent key is its default. A missing file is an
ordinary start; an unreadable or malformed file, an unknown key, or a value of
the wrong type or out of range is a notice in the top bar, and only the keys it
affects fall back. A Cell size that does not land on whole physical pixels is
snapped where it is drawn, and its Glyph follows the snapped Cell.

`style::install` registers `style()` of the selected dark Theme for
`egui::Theme::Dark` and of the selected light Theme for `egui::Theme::Light`
through `set_style_of`. egui chooses the appearance each frame from the mode
and the operating system's appearance, and the console paints the Source, and
answers the window's clear colour, from the Theme of that same appearance, so a
frame never takes its chrome from one Theme and its Source from another. A
mode change is applied once the frame it was made in is done, and the next
frame presents it.

The mode is egui's own `ThemePreference`, which eframe stores with egui memory;
nothing in the console forces it at startup, so a fresh start follows the OS.
The Theme selections are never stored: the `dark_theme`, `light_theme` and
`cursor_effects` storage keys earlier builds wrote are no longer read or
written.

A selection names a built-in or a custom Theme `07` loads. At startup the
native console reads `~/.orcvs/themes/`; the web console has the built-ins
alone. A
custom Theme's identity is its file name's stem, and its `name` is only a
display label. When a selection is missing, malformed, conflicted or of the
wrong appearance, a Theme notice in the top bar names the Theme, the setting
and the file problem, and the appearance's built-in is shown in its place; the
settings file is never rewritten, so repairing the Theme file presents it on
the next launch. The notices also name any loaded Theme with a text state that
fails the contrast floor, which a Comment does not; such a Theme still loads
and can be selected.

## Source colours

Each Source Paint role paints its foreground and background from the resolved
Theme: `source.ordinary` (also Char), `source.comment`,
`source.function`, `source.bang`, `source.number`, `source.note`,
`diagnostic.*` and `output_portal.*`, each with its own
`.background`. `Theme → Source colours`, its Fill tint slider and its "Reset to
theme defaults" are gone, and the old `source_paint` storage key is left
unread.

Each role background is its own stored colour, alpha included. There is no
`fill_tint` property: changing a role's foreground does not recalculate its
background. Every recognized Function Cell — nested
Functions included — and
every Operand Cell (its declared Token: Number or Note, whether the operand is still Pending, Valid, or Invalid) paints its role
background, and a Bang paints `source.bang.background`. Comment, an empty
unclaimed Cell, and a Leftover Char have a transparent one. A refused Function spelling takes none either (see
Diagnostic, below) — only a Function entry the Parser recognized does. Role
backgrounds composite over the uniform `cell.background`, which is transparent
in both built-ins. On the Cursor's own Cell, the Cursor's fill wins outright.
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
to the same one decision: whether a Cell draws as a value Function's Output
Portal (the Output Portal role in `RenderCell::portal_roles()`, `.scratch/syntax-
highlighting/issues/05`, `10` and `12`'s Answers) — the Cell pair the
Function's Reservation covers at its Output Portal, for a root and for a
nested Function alike, since a nested Function writes its own Output Portal
as well as returning its answer. All of it is known from the
current Source revision alone and so lit before any Tick runs. Parsing is
unchanged and unaware of it (`05`'s Answer): a written answer re-parses
exactly as ordinary
Source would (a `07` left south of `.+0304` is two unknown one-Cell
Functions, diagnostics included), and the Output Portal fact is what tells
that written value apart from the Expression that produced it. Such a Cell
draws in the Output Portal colour on `output_portal.background`
instead of whatever its Source Paint fact alone would answer; an empty Output
Portal Cell shows the same background with no glyph. The one named exception is a
Bang answer: it keeps all its own paint, because a Bang is what a Producer
emits rather than a value it writes, so under both built-ins it draws as a Bang
anywhere else does.

**Precedence where an Output Portal covers another Expression's claimed
Cells** — a consumer's operand, or another root, per `05`'s Overlap rule that
the fact "covers every Cell of the Reservation whatever else claims
it": a Cell that is itself a bound Function's own two-Cell spelling keeps its
Function paint outright, root or nested alike, because every Function's own
spelling already carries `Token::Function` regardless of nesting and telling
a root's spelling from a nested one would need the Expression this decision
does not read. Every other overlapping Cell — another root's own Number or
Note operand among them — takes the Output Portal colour
and background instead of its own declared role, because the root's answer
is what a viewer reads there. The Cursor's own fill still wins outright over
everything above, on its own Cell.

`console/src/contrast.rs::validate` (`.scratch/theming/issues/08`) measures
every reachable painted text state's *effective*, actually-composited
foreground against its actually-composited background — asking
`style::CellColours::colours`, the same decision painting itself asks, so the
two cannot independently drift.
Each result reports its role, its state (a `CursorPlacement` of `plain`,
`Cursor`, `Region` or `Region, Cursor's Cell`, crossed with the Portal roles
the Cell plays: none, Input Portal, Output Portal, or both), the effective foreground and
background, and the measured ratio against the 4.5:1 floor. Every role but Comment is held to the floor: a Comment
state below it is reported and does not fail (`ContrastResult::fails`). The floor is stated once, at `contrast::CONTRAST_FLOOR`,
from WCAG 2.1 Success Criterion 1.4.3 ("Contrast (Minimum)"), and the returned
`ContrastReport` carries it too; the scope is stated once, in the module's
`# Scope` section. `validate` measures text contrast only: never Token-colour
distinguishability or border/focus visibility.

Colour vision is a second, test-only measurement over the built-in Themes,
`contrast::colour_vision::distinguish` (`.scratch/theming/issues/04`), added
because a definition can clear every one of those states and still paint two
Source glyph channels a dichromat reads as one colour. It simulates protanopia, deuteranopia and
tritanopia (Viénot, Brettel & Mollon 1999) over the same composited colours,
measures CIEDE2000 between every pair of glyph channels, and reports the closest
placement for each pair. `contrast::colour_vision::CONFUSION_FLOOR`, 5.0, gates
the two red–green dichromacies; tritanopia is reported and not gated, for the
reason `ColourVision`'s own documentation states.
`contrast::colour_vision::tests::shipped_theme_colour_vision_gate` runs it over
every built-in; no shipped path runs it, so a loaded Theme document is not
measured for colour vision. It measures glyphs only: never background tints
against each other, and never anomalous trichromacy.

A Pending operand Cell draws no glyph, so `validate` has no Pending role to
measure — `Role` (Number, Note) carries Valid and Invalid only.

`contrast::tests::shipped_theme_gate` runs over every built-in the registry
holds: every state of a role held to the floor clears 4.5:1, and a Comment
below it is reported and does not fail. There are no accepted exceptions: a
state below the floor of a role held to it fails the gate.

The console chrome's states (`.scratch/theming/issues/11`) are read from
`style::style` — the `Style` `style::install` registers — through the egui
accessors the widgets paint from (`egui::Style::button_style` for a button's
fill and text, `widget_style` for a frameless label, `Visuals::selection` for
selected text), and composited over the surface each is painted on down to
the window backdrop, so the report and the chrome's painting read one
mapping. Left
out, as `contrast::validate`'s own scope states: disabled widgets (egui fades
them, and WCAG 2.1 SC 1.4.3 exempts inactive components), `code.background`
(nothing paints a code span), a widget state's strong fill (egui paints no
text on it), a popup or window floating over another surface — the Source
Grid or a panel — (measured as one panel over the window backdrop, identical
while `panel.background` is opaque, as both built-ins' are), and `text.active`
on the open widget fill (egui paints an open widget's text in
`widgets.open.fg_stroke`, which `style::style` maps from `text`, so the open
fill is measured with `text`, the pair actually painted).

The single-Cell Cursor always replaces a role's background outright, whatever
that role's own background's own alpha — `style::cell_visuals_with_
cursor_colour`'s "the Cursor's own fill wins outright on its Cell." A Region
Cell, by contrast, only falls back to the Region fill when the role's own
background left nothing painted at all (alpha exactly `0`) — a role
background with any nonzero alpha, translucent or opaque, keeps winning
there.

Sector boundaries are solid lines drawn over Cell edges, along the whole of
every sector edge, in the Theme's `sector.seam` at `sector.seam.width`: styled
the way a Cell grid line is styled by `grid.border` and `grid.border.width`.
They replace the historical `+` Marker Glyphs, leaving every empty Cell visually
empty while preserving the configured Sector Seam spacing as geometry.
