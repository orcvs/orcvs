# Which glyph shows an empty Cell

Status: needs-triage

## The question

How should an empty Cell look, in the console and in a Source File? Today it is a space in both.
Orca, and Orcvs until July 2024, used `.`. The user finds `.` has a certain literal elegance on a
Grid, but has not decided. This effort exists to put the options side by side and record the
choice.

## What is not in question

The semantics are settled, and every option below keeps them unchanged:

- An empty Cell is a character like any other (ADR 0033). It sits between Expressions at no cost,
  and inside a claim it is Source that nobody has written.
- Copying Functions move empty Cells as they move any other character. A Copy whose aligned input
  is empty clears its destination, and a Track whose selected pair is empty copies its two empty
  Cells (ADR 0067, ADR 0069).
- An operand slot whose Cells are all empty is invalid input: the Function does not evaluate,
  writes nothing, returns nothing and is diagnosed, as a partly written slot is (ADR 0069).

Only the glyph is open: how those rules look on the Grid and in the file.

## History

- **Orca** draws an empty Cell as `.`, and draws its background rulings as characters as well: a
  `+` at every Marker Spacing, and a `.` dot at the Highlight Spacing.
- **The 2024 console prototype** followed Orca, with `const TERMINATOR: &str = "."` in
  `console/src/source.rs`.
- **`cf37d1be`** (2024-07-13, "replicating orca grid") changed it to
  `pub const TERMINATOR: &str = " "`, and from then on an empty Cell has been a space. The commit
  records no reason. The likely one is that the same change began writing Orca's background
  decorations (`+` markers, `.` dots) into the Source string, and empty needed to look different
  from them.
- **ADR 0008** (added in `30370e11`, 2026-08-31) gave `.` to the numeric family: `.+ .- .| .x ./ .%
  .< .> .= .v .^`. The Clock spelling `~.` uses it as an operation glyph too. By then `.` had not
  meant "empty" for two years.
- **ADR 0034's Render Frame amendment** (`cc956fe0`, 2026-09-13) made the Marker and Highlight
  decorations painted geometry instead of Cell content. That removed the conflict `cf37d1be`
  probably worked around. The same amendment, delivered by `syntax-highlighting/03`, decided that
  an empty claimed operand Cell shows its declared Token's tint and no character.
- **`orcvs/src/source/file.rs`** sets the Source File format: one line per row, a space for an
  empty Cell, trailing spaces trimmed when the file is written.

## Options

### 1. Space, drawn as a space (status quo)

Source, Source File and console all use a space.

- Cost: none.
- What the Grid shows: unwritten operand slots are visible only through their Token tint. In plain
  text (a file, a diff, a terminal, an ADR code span) an unwritten `.+01` slot and a finished one look
  the same unless the trailing spaces survive. An empty pair Track reads is two spaces, which reads as
  separation rather than as a rest.

### 2. Space in Source and file, `.` painted by the console

The Source and the Source File are unchanged. The console paints a dim `.` in every empty Cell.
This is presentation only, the same move ADR 0034 made when it began painting Orca's rulings as
geometry instead of storing them as characters.

- Cost: small and reversible. It is a paint rule in `console/src/paint.rs`, plus a decision about
  how the dot sits alongside the painted sector seams and the Cursor bloom.
- Sub-choices: whether claimed operand Cells get the dot as well as their tint, and whether Comment
  text keeps its spaces (the painter knows the Comment's claim, so it can).
- Risk: a painted `.` and the numeric family sigil `.` are the same glyph. `..+01..` is legible
  only while the painted dot is clearly dimmer than Source. In monochrome, or to someone reading
  the console aloud, they collide. Files, diffs and documentation still show spaces, so the
  console and the text disagree.

### 3. `.` is the Source character for empty

An empty Cell holds `.` in Source, in the file and on screen.

- Cost: large, and it changes the language.
  - Respell the whole numeric `.` family and the Clock `~.`. ADR 0008 and at least 16 ADR and
    `CONTEXT.md` files quote those spellings in code spans, and about 30 files under `lang/src`,
    `orcvs/src` and `console/src` spell them in string literals. The parser, the Function table, the
    Function reference and every test fixture that spells a numeric Function all change.
  - Change the Source File format in `orcvs/src/source/file.rs`. Empty is stored as `.`, and the
    writer must decide whether it still trims trailing empties and whether a space in a file is
    refused or read as empty.
  - Every saved Source File changes meaning. Under the old reading, a file that uses `.+` contains
    empty Cells where the numeric Functions were.
  - Comment text loses its spaces, because a space in a Comment is an empty Cell and is drawn as `.`.
- Gain: the file, the diff and the screen agree. An empty pair Track reads is visibly `..`, and an
  unwritten operand is visibly unwritten in plain text.
- This option needs a replacement numeric sigil, which is itself a design question. The prototype
  uses `#` (and `~:` for Clock) as a **placeholder** only, so the Grids can be drawn.

## Deliverable

`issues/01-prototype-the-empty-cell-glyph.md`: a syntax prototype that draws the same Source Grids
under all three options, so the user can choose by looking at them. The follow-up issues depend on
which option is chosen and are filed once it is.

## Comments

### Audit against origin/main cf01e3c — 2026-10-13

"What is not in question" cited ADR 0066's pending state, which ADR 0069 superseded, and ADR 0063's
Lists and Items, which ADR 0067 superseded; it now states the current rules, and the options use
the same vocabulary. The prototype the Deliverable names was never committed (`e1c1643` added only
the ticket and this spec). The glyph question itself is unchanged.
