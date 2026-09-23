# Orcvs Theme format, version 1

Status: implementation specification, prepared 2026-09-22 from the confirmed
product decisions in [ADR 0053](../../docs/adr/0053-one-theme-styles-the-whole-console.md).
This completes the earlier schema draft. It specifies the dark built-in and file
contract; it does not claim runtime implementation or approve the light palette.

## Document

```yaml
format: orcvs-theme
version: 1
name: "My Dark"
inherits: "okabe-ito"
style:
  text: "#E0E0E0"
  grid.background: "#101820"
  grid.border: "#35506080"
  grid.border.width: 0.5
  cursor.background: "none"
```

Required root fields are `format`, `version`, `name` and `inherits`. `style` is
optional; omission or `style: {}` inherits the complete parent unchanged.
Optional `appearance` is `dark` or `light` and must match the parent. The filename
stem is the identity; `name` is a nonempty display label. `okabe-ito` is the reserved
dark built-in identity, `orcvs-light` the reserved light identity. The latter's
palette must pass the separate review gate before switching is exposed.

Only this native format is supported. No palette slots, Base16 import, tint
scalar, general expressions, font assets or in-app authoring are required.
Unsupported format/version and unknown root/style keys reject the whole document.

## Property catalogue and dark definition

Every row is a named `style` property. Built-ins define every property explicitly;
custom documents replace only supplied properties. Property names are case-sensitive,
including dots, which are literal characters rather than nested object access.
All colour values use straight RGB/RGBA hex. Six digits mean opaque. `none` is a
string accepted only for the two optional Cursor fills; it is not YAML null.

`source.ordinary` supplies Ordinary, Char and Atom foregrounds. Atom background
is separate because the existing declared Atom operand has a fill. Char has no
independent paint fact and uses the ordinary background; no unused Char-only
property or production test seam is introduced.

| Colour property | Okabe–Ito value |
|---|---|
| `window.background` | `#0B1112` |
| `panel.background` | `#0B1112` |
| `grid.background` | `#000000` |
| `cell.background` | `#00000000` |
| `source.ordinary` | `#EAEBE5` |
| `source.comment` | `#999999` |
| `source.number` | `#56B4E9` |
| `source.note` | `#F0E442` |
| `source.function` | `#009E73` |
| `source.bang` | `#CC79A7` |
| `source.sequence` | `#0072B2` |
| `source.ordinary.background` | `#00000000` |
| `source.comment.background` | `#00000000` |
| `source.number.background` | `#56B4E91A` |
| `source.note.background` | `#F0E4421A` |
| `source.function.background` | `#009E731A` |
| `source.bang.background` | `#00000000` |
| `source.atom.background` | `#EAEBE51A` |
| `source.sequence.background` | `#0072B21A` |
| `diagnostic.foreground` | `#D55E00` |
| `diagnostic.background` | `#00000000` |
| `diagnostic.border` | `#00000000` |
| `output_portal.foreground` | `#E69F00` |
| `output_portal.background` | `#E69F001A` |
| `output_portal.border` | `#00000000` |
| `grid.border` | `#1D373148` |
| `sector.seam` | `#3765566E` |
| `cursor.border` | `#EAEBE5` |
| `region.border` | `#EAEBE5` |
| `cursor.area` | `#4CBE9C` |
| `cursor.background` | `none` |
| `region.cursor.background` | `none` |
| `region.background` | `#FFFFFF2B` |
| `panel.border` | `#1C3932` |
| `selection.background` | `#0A2A22` |
| `selection.border` | `#65E6BE` |
| `selection.border.rest` | `#52C3A3` |
| `widget.inactive.border` | `#1C3932` |
| `text` | `#EAEBE5` |
| `text.active` | `#65E6BE` |
| `text.muted` | `#E9EBE499` |
| `input.background` | `#000000` |
| `link` | `#5AAAFF` |
| `code.background` | `#404040` |
| `input.cursor` | `#C0DEFF` |
| `error` | `#CC79A7` |
| `warning` | `#CC79A7` |

| Width property | Default, display points | Inclusive range |
|---|---:|---|
| `grid.border.width` | 0.5 | 0–1 |
| `sector.seam.width` | 0.75 | 0–1 |
| `cell.selection.border.width` | 0.5 | 0–1 |
| `cursor.border.width` | 1 | 0–1 |
| `region.border.width` | 1 | 0–1 |
| `diagnostic.border.width` | 0.5 | 0–1 |
| `output_portal.border.width` | 0.5 | 0–1 |
| `panel.border.width` | 1 | 0–2 |
| `selection.border.width` | 1 | 0–2 |
| `widget.border.width` | 1 | 0–2 |
| `widget.inactive.border.width` | 0 | 0–2 |
| `input.cursor.width` | 2 | 0–2 |

Zero width suppresses that stroke, not fills or other strokes. Diagnostic/Portal
width defaults to 0.5 while its transparent colour preserves the current absence.
Inactive widget width zero preserves its current absence. The 2-point text-input
caret is separate from ordinary 1-point chrome borders. Widths do not scale with
Grid zoom; physical-pixel snapping follows existing viewport rules. Cursor/Region
fragments retain 0.45–1.25 nominal-width modulation. Nominal width is bounded by 1,
so a fragment can reach 1.25 points; width zero produces no fragments.

The opaque root is `window.background`; alpha other than 255 on this property is
an error. Panel, Grid and Cell surfaces can have alpha. The new root value reuses
the existing page colour. Role background values are each tinted role's own
foreground colour at a uniform 10% opacity (`0x1A` alpha), a 2026-09-22 retune of
the values first extracted from the old 16% gamma-byte interpolation over black.
They are literal inherited colours, not a runtime formula — the 10% figure is
baked into each stored value, not a shared `fill_tint`-style scalar applied at
paint time. Foreground edits do not change them.

`text.muted` round-trips to existing premultiplied bytes `[140,141,137,153]`.
It preserves egui's existing global weak-text colour. The chrome border
`#1C3932` preserves the actual conversion from the premultiplied Grid colour;
it is not the source comment's approximate hue `#1D3731`. It is not a value recomputed
from a custom Theme's `text`. Error and warning likewise retain their existing
built-in colours but no longer borrow the live Bang property.

## Chrome mapping

| Consumer | Properties/derivation |
|---|---|
| Application backdrop | `window.background`, opaque |
| Panel/window fill; normal widget fills | `panel.background` |
| Source surface | `grid.background` |
| Extreme/faint input background | `input.background` |
| Panel/window/noninteractive border | `panel.border`, `panel.border.width` |
| Inactive widget border | `widget.inactive.border`, `widget.inactive.border.width` |
| Hovered/open widget border | `selection.border.rest`, `widget.border.width` |
| Active widget border | `selection.border`, `widget.border.width` |
| Hovered/active strong and weak fill; open strong fill | `selection.background` |
| Open widget weak fill | `panel.background` |
| Normal/open/noninteractive widget foreground | `text` |
| Hovered/active widget foreground | `text.active` |
| Weak text and input hints | `text.muted`, explicit, no further weak attenuation |
| Text selection | `selection.background`, `selection.border`, `selection.border.width` |
| Input caret and active IME underline | `input.cursor`, `input.cursor.width` |
| Inactive IME underline | Same width; existing half-linear colour attenuation |
| Links; code spans | `link`; `code.background` |
| Error/warning messages | `error`; `warning` |

Disabled widget opacity remains the existing 0.5 toolkit operation applied to
Theme-derived colours. Icon/foreground stroke geometry, corner radii, layout,
fonts, text sizes and widget expansion remain existing values. These are not
additional independently configurable controls. Any additional live toolkit
colour use discovered in implementation must be mapped to these properties or
added explicitly to this catalogue, never silently left as a second palette.

## Source composition and inheritance

1. Resolve a complete built-in. Custom documents copy its resolved properties,
   then replace explicit values. Only built-ins can be parents. Parent chains,
   missing parents and conflicting appearance declarations are errors.
2. Composite panel/Grid surfaces over the opaque root. Composite uniform
   `cell.background` over the Grid inside every Cell; it is not a fact fill.
3. Select Source role from existing paint facts. Declared Number/Note/Atom/Sequence
   operands retain their role background in Pending, Valid and Invalid states.
   Invalid foreground uses the Diagnostic channel over the declared role; a
   transparent Diagnostic channel reveals the declared role. Pending glyphs stay
   blank. Function, Comment, Bang and Ordinary use their respective properties.
4. Portal channels take precedence over other non-Function facts, including
   Diagnostic. A bound Function retains all its own paint inside a Portal. Bang
   retains its glyph foreground and takes Portal background. Transparent Portal
   channels reveal the underlying role under these fixed exceptions. Border
   channels use the same fact priority and compose over the ordinary Grid border.
5. Cursor/Region handling retains existing precedence. A single selected Cell
   suppresses ordinary role fill in favour of its optional Cursor fill. In a
   multi-Cell Region, the Cursor uses Region Cursor fill, falling back to ordinary
   Cursor fill when absent. Other Region Cells use Region fill only when no visible
   effective fact fill exists after role, Diagnostic and Portal composition. A
   transparent role background alone does not hide a visible fact overlay;
   uniform `cell.background` never prevents this fallback. Selected Cell borders
   apply only to a single-Cell Cursor, which also suppresses Sector Seams. They
   use `selection.border`/`.rest` and `cell.selection.border.width`. In a multi-Cell
   Region, its Cursor Cell keeps the ordinary Grid border and Sector Seams; the effect
   outline uses `cursor.border` or `region.border` and its separate width.
6. Composite foreground/background/border channels with alpha using the same
   pinned colour operations as painting. Contrast reports use these exact resolved
   foreground/background pairs, not raw theme hex pairs. Preserve actual overlap
   cases and validate the normal-zoom dark result against current paint output.

Omitted optional-fill properties inherit. String `none` removes that optional fill
and enables the fallback above. Transparent supplied colour suppresses fallback;
it remains distinct from absence. The uniform base layer still shows beneath it.
The single-Cursor `none` case does not restore the ordinary role fill it suppresses.

Compute finite fact combinations and colour composites outside the per-Cell loop.
No per-Cell string lookup, hashing, YAML access or colour interpolation is allowed.
Cursor area alpha must propagate through tears/strands, including complete hiding
at zero; do not reinterpret premultiplied bytes as straight RGB.

## Parser subset and limits

Accept UTF-8 with optional initial BOM, LF or CRLF, blank lines and comments outside
strings. Root entries start at column zero. Nonempty `style` mappings use exactly
two leading spaces; tabs for indentation are rejected. Only a root mapping plus
one mapping of literal style keys is supported. Optional initial `---` is accepted;
further document markers, sequences, anchors, aliases, tags, merge keys and
multiline scalars are rejected. An empty mapping may be written only as `{}`.

String values may use single or double quotes. Single quotes escape an apostrophe
by doubling it. Double quotes support `\\`, `\"`, `\n`, `\r`, `\t` and four-hex-digit
`\u` escapes, pairing UTF-16 surrogates correctly; unsupported escapes are errors.
Names/identities must decode to nonempty strings with no control characters.
Plain `orcvs-theme`, `dark` and `light` are allowed for their header fields.
Colours and `none` must be quoted. A comment starts with `#` outside a string,
after whitespace; a quoted hex prefix is never a comment.

Version syntax is the integer `1`. Width lexical syntax is `[0-9]+(\.[0-9]+)?`:
no signs, exponents, separators, NaN or infinity. Parse to finite numeric values,
then check inclusive property bounds. Reject duplicate keys even if values agree.
Colours are exactly `#` plus 6 or 8 hexadecimal digits; normalize channels, not
property/identity case. Decode the document completely and validate before registry
mutation. Error reports identify filename, line and offending field where available.

Set a 1 MiB per-document byte limit before allocation/read completion. Check both
native files and web blobs, and recheck actual bytes. Limit decoded display names
and parent identities to 256 UTF-8 bytes; error rather than truncate. The fixed
key catalogue bounds map entries; duplicate keys fail immediately. These are
implementation resource limits, not a second format or extensibility mechanism.

## Discovery, persistence and failures

Scan direct `.yaml` and `.yml` children of `~/.orcvs/themes/`, case-insensitive
suffix matching, without recursion. Missing directory means no custom Themes.
Report unreadable files/directory without preventing startup. File symlinks may
be followed subject to the same byte limit; do not traverse directory symlinks.
Use the discovered filename stem as identity, case-sensitive; reserve built-in
identities. Detect duplicate stems before loading, reject all conflicting files,
and report their paths. Directory enumeration order cannot choose a winner.
Ignore unrelated suffixes. An empty stem is invalid.

Read/parse native files during initialization outside UI render callbacks. File
changes take effect next launch only; native application storage never supplies
cached Theme values. Web file import reads bytes asynchronously through existing
platform facilities, parses the same format, then atomically replaces a document
of the same identity if valid. Cancellation leaves registry/selection unchanged.
With persistence, web retains imported source documents and selections; without
it these remain session-only. Failed storage writes are reported rather than
claiming persistence succeeded; the valid in-memory Theme may remain usable.

Unavailable, malformed, conflicted or appearance-mismatched selections keep their
saved references and show the matching built-in fallback plus an error. Autosave
must not overwrite that reference. This applies to corrupted stored web documents
and a valid reimport that changes appearance as well as native failures. Repair
restores the intended Theme when next loaded. Contrast warnings alone do not
reject user documents. Imports do not silently change the selected identity;
a selected same-identity reimport updates the effective Theme coherently.

## Acceptance and implementation boundary

Use this catalogue for `06`, `03`, `07` and `10`. `06` owns the document model,
built-in values and pure inheritance resolver; `07` owns parser and transports;
`10` verifies/documents the complete custom file workflow. No second custom parser.
The benchmark floor is an existing prerequisite; compare it in CI.

Regression coverage must exercise every field's production consumer, role and
channel overlaps, exact dark values, weak/IME colours, zero/partial alpha, uniform
Cell backgrounds, optional fills, zoom/DPI widths, parser boundaries, invalid
identities, persistence-off builds and fallback/save/repair. Property tests for
parser/colour boundaries follow repository policy; local proptest cases are 32.
Tests must use shipped paths without adding test-only inputs to production APIs.

There is no claim of full accessibility. Pairwise distinguishability is
validated only for the Source glyph channels under a simulated red–green
dichromacy (`contrast::distinguish`, `.scratch/theming/issues/04`); tritanopia
is measured but not gated, and normal-vision distinguishability is not
measured.

### Known dark failures, resolved by the 2026-09-22 tint retune

Every previously known below-floor state — the invalid Number, Note and Atom
operand Diagnostic states, and Sequence's own tint — was resolved by retuning
every tinted role background to a uniform 10% opacity (each role's own
foreground colour at alpha `0x1A`), replacing the previous precomputed opaque
tints. Every reachable state `console/src/contrast.rs::validate` measures now
clears the 4.5:1 floor; `contrast::tests::shipped_theme_gate` runs as a real,
non-`#[ignore]`d test and its accepted-exception list is empty, because there
is nothing left to except. `console/src/theme.md` records the exact measured
figures. No floor was lowered and no measurement was special-cased to reach
this result — the retune changed the colours the floor is measured against.

The light palette is deliberately not invented here. Issue `04` produces the
complete `orcvs-light` definition, console captures and contrast results for user
review. Switching remains unexposed until that acceptance and coherent Source/
chrome rendering. Through `03`, retain `02`'s shared presentation: register the
same resolved Okabe–Ito style in both egui appearance slots and render Source
from that same Theme, preserving saved preferences. `04` owns replacing this
with distinct registration and the switching acceptance tests.

This is a specification, not a passing implementation. Runtime gates and visual
captures are owed by the implementation; documentation validation alone does not
establish rendering fidelity. Font choice and broader geometry remain unplanned.
