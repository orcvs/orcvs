# Which glyph delimits a Comment

Status: needs-triage

## The question

Which delimiter should open and close a Comment? Today it is `||` (ADR 0035), and a Comment
closes at the next `||` on its row (ADR 0074). The tracker example frames each pattern row in one
Comment so its Notes are text the Reads select. Toby finds the doubled bar hard to read on a Grid,
in the row and in the step marks inside it, and asked on 2026-10-20 for the alternatives.

## What is not in question

The Comment's semantics stay as ADR 0035 and ADR 0074 decide them, in every option:

- It opens only where an Expression could start, and closes at the next delimiter on its row, or
  claims the rest of the row when none follows.
- Its text is ordinary Cells to every Function: a Read, Track or Copy selecting a pair of it reads
  that pair as data.
- Only the delimiters are occupied Cells. It answers no value and is never scheduled.

Only the spelling of the delimiter is open.

## History

- **ADR 0023** spelled the introducer `##`, two Cells, so every reserved spelling was recognised in
  two-Cell units, and one `#` alone was incomplete Source.
- **ADR 0035** moved it to `||`, because `#` belongs to the Note domain (`:#` was Note Range) and a
  doubled glyph is the operandless family beside `**` and the Activations.
- `:#` and the Sequences were retired by ADR 0063, and no Note spelling uses `#` (sharps are
  lowercase, `c4`). So `#` is unused today.
- Probed against the Parser on 2026-10-20: `##` and `;;` each analyse as
  `unknown function "##"` and `unknown function ";;"`, so neither is a spelling today.
- **ADR 0079** (2026-10-21) draws each Cell 12 points wide and 18 tall in Monaspace Neon, no
  longer a 16 point square, so a doubled glyph's two marks sit closer together than they did when
  this question was asked. The prototype draws Cells at that shape.

## Options

1. **`||`**, unchanged.
2. **A different doubled glyph**: `##`, `;;`, `::` or `--`, two Cells as every unit is today.
   Each needs the introducer changed in the Parser and every Source File and document that
   spells a Comment.
3. **One-Cell `#`**, as Orca writes it. This is the first one-Cell unit, so it reopens ADR 0023's
   two-Cell recognition rule and ADR 0035's reservation of `#` for pitch.

Step marks inside a Comment, such as `[C4]`, are Comment text and need no language change under
any option. They are compared in the same prototype because they decide how a row reads.

## Decision

Pending issue 01's prototype.
