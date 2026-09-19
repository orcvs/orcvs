# The primary drag selects a Region

Status: accepted. Extends [ADR 0045](0045-the-source-view-is-a-bounded-space.md), whose Pan gestures stand.

The console gains a Region: a rectangle of Positions within the Grid, spanned from an anchor Cell to a live end, with the Cursor on one of its Cells. A Region always exists; when the anchor and the live end are the same Cell it is that one Cell, which is the ordinary state. The Cursor stays one Cell, so every rule that follows the Cursor — the Pan that brings it into view, the Cursor Effect — still names one Cell. It sits at the live end, except after command `A`, which spans the whole Grid around the Cursor and leaves it where it was, as a spreadsheet leaves its active Cell.

**The primary drag selects; Alt, the middle button and the wheel Pan.** A primary press sets the anchor and the Cursor on the Cell under the pointer; dragging moves the Cursor to the Cell under the pointer; release keeps the Region; a click collapses it; Shift with a click extends it from the anchor it already has. A drag past the console's edge moves the Cursor past the edge, and the Cursor follow of ADR 0045 scrolls the Source View after it — faster the further the pointer is outside, never more than one Cell a frame, and never past the Grid. Tiled makes the same split: a primary drag belongs to the tool, and Alt with a primary drag or a middle drag Pans.

**The keyboard follows the spreadsheet.** Shift with an arrow moves the live end and keeps the anchor, and the Cursor moves with the live end; a bare arrow collapses the Region and moves; command `A` spans the whole Grid and leaves the Cursor where it was, so the Source View does not move and the next keystroke writes where it would have; Escape collapses the Region onto the Cursor. After command `A` the live end is the Grid's last Cell, so the first Shift with an arrow moves that corner and the Cursor rejoins it there. Moving the live end past the anchor flips the rectangle, because the Region is the span between the two and has no corner of its own.

**A Region is written as a block.** Typing writes at the Cursor, steps right, and collapses the Region, as typing does today. Command Enter followed by a character fills every Cell of the Region with it. Backspace and Delete empty every Cell of a Region larger than one Cell and keep it; on one Cell they do what they do today. Copy puts the Region's rows on the clipboard as text joined by newlines, keeping trailing spaces so the shape survives; cut copies and then empties. Paste writes from the Region's top-left, spaces included, clipped at the Grid's edges, and the Region becomes the rectangle that landed. A character the clipboard carries that cannot be a Cell lands as an empty Cell, and `\r\n` is one row break.

## Rejected alternatives

**A Cursor with an extent.** Orca and Orca-c give the Cursor a width and a height. That redefines the Cursor for every rule written about its one Cell, and it leaves open which corner the Cursor is drawn on: Orca-c draws it at the top-left and Orca at the press point, and neither follows the pointer, so a drag past the edge would need a scroll of its own.

**Typing fills the Region.** Orca-c fills on every keystroke. Orcvs has no undo, and command `A` followed by any character would overwrite the whole Grid with no way back; the fill is kept behind command Enter, which is Excel's gesture for the same thing.

**Command `A` moves the Cursor to a corner.** Keeping the Cursor at the live end everywhere would need no second corner, but command `A` would then move the Cursor to a corner of the Grid and the Cursor follow would scroll the Source View there. Every spreadsheet leaves the active Cell where it was, and the view with it.

**A bare arrow moves the Region.** Orca translates the rectangle on a bare arrow. Every spreadsheet collapses it, and moving a Region's contents is a separate operation that has not been designed.

**A primary drag Pans.** egui tells a click from a drag by distance, so a primary drag could Pan with no modifier. It would spend the one gesture a Region has, and the wheel, the two-finger scroll and the middle drag already Pan without it.

**Option drag as column selection.** macOS text tools take Option with a drag for a rectangular selection. Every Region is already rectangular, so the modifier has nothing to select and is free to Pan.

## Consequences

`CONTEXT.md` gains Region. The Region lives beside the Cursor in `Orcvs`, reaches the console on the Render Frame, and is not stored with the Source. A Region larger than one Cell is drawn with a background tint; a Region of one Cell draws none.
