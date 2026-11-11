# 01 — A stray MIDI output nests the Expression beside it

Status: needs-triage

Type: research

Blocked by: None — can start immediately

**Question:** should a MIDI output whose leading operands are empty take a Function written in a
later operand slot as nested, silently, or should the performer be told?

Under ADR 0106 an operand of two empty Cells is a valid empty operand. So `!>    .+0102`, a Raw Play
typed to the left of an existing Addition, reads as a Raw Play with an empty channel and velocity
and `.+0102` nested in its note, exactly as `!>017F.+0102` nests it. The row sends nothing, because
the channel is empty, and raises no diagnostic. `.+0102` still takes its Turn, but it is no longer
its own root, and nothing tells the performer. Before ADR 0106 the empty channel was invalid input,
so the row was diagnosed. The same holds for `!c` and `!b`, and for any output whose empty operands
come before the slot a Function lands in.

Settled for now on PR #373: keep the behaviour, which follows from the rule that an output sees only
its Cells and a Function written where an operand stands is nested there.

## To investigate

- How often the case arises in Live Editing: typing an output's spelling to the left of an existing
  Expression, or deleting an output's operands back to empty.
- Whether a hint that is not a refusal fits, for example Source Paint or a Language Map note that a
  root became nested under an output that sends nothing.
- What any refusal would cost. A rule such as "an output with an empty operand refuses a nested
  Function" makes one operand's meaning depend on another, which ADR 0106 removed.
- Orca has no equivalent, because its operators do not nest.
