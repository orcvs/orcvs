# 05 — Write a value in a direction or at a Position

Status: resolved
Blocked by: 02 — Read a pair in a direction; 04 — Decide how a dynamic Output Portal is ordered; 07 — Accept the Read, Write and Copy families; 10 — Read a pair at a Position

**What to build:**

The Write family of ADR 0070: the directional Writes `@^ @v @< @> n value` and the absolute Write `@$ column row value`, covering Orca's `X`. Each writes `value` to the pair its operands address: a directional Write counts exactly as the Read with the same arrow does (ticket 02), and `@$` addresses the Position the absolute Read does (ticket 10). Writes are ordered as ticket 04 decides. Orca's Generator is a Read nested in a Write's `value` operand, with no separate Function.

## Acceptance criteria

- [x] `@^`, `@v`, `@<`, `@>` and `@$` are in the Function table with Number address operands and an untyped `value` operand; an unwritten operand makes the Write invalid (ADR 0069).
- [x] `value` carries its encoding unchanged (ADR 0070's amendment, "a Write carries its value"): a literal Note, Number or `**` is written as spelled, `**` as a Bang at the destination; a nested Function's Return is written as returned, so a Read nested in `value` passes on a Note, a Number, a Function spelling (which replaces the Function at the destination) or a Bang; only the destination's reader decodes it. A `**` in a typed operand stays invalid syntax.
- [x] The Source Paint paints a `value` literal as plain Source text whatever its Cells spell: it has no type of its own, and the Writes add no paint, colour or Theme role.
- [x] A dynamic write's Bang that reaches a root which has taken its Turn, and a lock from a Halt that joined the Tick that reaches a computation which has taken its Turn, are missed and diagnosed as missed at the target, not as a cycle (ADR 0067's amendment); nothing is stopped, and the Write's other effects, the `**` display and the activation of other roots still to take their Turn, still apply.
- [x] A directional Write with a given arrow and `n` addresses the same pair, relative to its anchor, as the Read with that arrow and `n`; `@$` addresses the same pair as `&$` with the same Position.
- [x] `@x 00 value` writes onto its own `n` operand, so on the next Tick it reaches a different distance; a test pins this self-stepping write.
- [x] A Write answers `value` through its dynamic Output Portal and writes nothing one row south; nested, it returns `value` to its parent, so `@$0000@$0101C4` writes `C4` at both Positions.
- [x] A destination outside the Grid or cut short by the row edge diagnoses and writes nothing, and a nested Write still returns `value`.
- [x] Readers of the destination that do not feed the Write see its write in the same Tick wherever they sit in the Grid, static, dynamic and nested readers alike; a reader that feeds the Write, such as the nested Read in the counter `@$0602.+&$060201`, sees it on the next Tick.
- [x] Two independent Writes reaching the same pair take their Turns in Grid order and the later write wins; a static writer that does not feed a Write writes after it.
- [x] A Write onto a Function's spelling suppresses or replaces it in the same Tick when that Function's Turn is still to come, and from the next Tick when it has been taken.
- [x] A Write that finds at its Turn a static writer it must wait for lets that writer go first rather than diagnosing a cycle; a cycle through a real dependency is diagnosed under ADRs 0065 and 0068.
- [x] A Read nested in a Write's `value` copies one pair to another place within a Tick, covering Orca's Generator.
- [x] A Source holding a Write gives the same result from a reused schedule as from a fresh one, for a write onto an operand and for a write onto a Function's spelling.
- [x] The Function reference names the Writes and describes Generator as the composition. ADR 0019's table and the glossary already do, from ADR 0070's acceptance.

## Resolution

The Writes are built as ADR 0067's amendment on dynamic Output Portals and ADR 0070's amendment on a Write's value decide, including the missed diagnostic for a Bang or lock that reaches a Turn already taken. A `value` literal paints with the default plain Source text: the user ruled that this work adds nothing to rendering.
