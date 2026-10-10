# 02 — Clear the output of a Bang producer that does not bang

Status: done

**What to build:**

ADR 0071's second decision. Equality `.=`, Delay `~*` and Euclidean `~%`, whose answers are only Bang and the Absence Marker, clear their Output Portal's two Cells on a Turn that answers the Absence Marker, as root and as nested Functions. A nested one still returns nothing to its parent. Every other Function keeps writing nothing for an absent answer.

## Acceptance criteria

- [x] First confirm the Cells written are already reserved for each of the three, root and nested, so the change adds no dependency edge. If they are not, stop and record what the schedule would need under `## Comments`.
- [x] A root Equality with unequal operands clears the pair one row south; with equal operands it writes `**` there and activates the aligned roots, as today.
- [x] Delay and Euclidean clear their Output Portal on every Tick they do not bang.
- [x] A nested Equality answering the Absence Marker clears its own Output Portal and leaves its parent invalid, as `a_nested_absence_marker_fails_its_parent` (`orcvs/src/source/tick.rs`) already pins for the parent.
- [x] A Function outside the three that answers the Absence Marker still writes nothing; `the_absence_marker_needs_no_destination_and_preserves_source` (`orcvs/src/source/model.rs`) stays green for it.
- [x] CONTEXT.md's Absence Marker, Equality, Delay and Euclidean entries state the clearing.
