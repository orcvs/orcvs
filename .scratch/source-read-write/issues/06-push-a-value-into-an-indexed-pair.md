# 06 — Push a value into an indexed pair

Status: resolved
Blocked by: 04 — Decide how a dynamic Output Portal is ordered; 05 — Write a value in a direction or at a Position; 07 — Accept the Read, Write and Copy families; 09 — Move Track to the Read family

**What to build:**

Push, `@t index count value`, the write counterpart of Track and Orca's `P` (ADR 0070). It writes `value` into the pair `index % count` pairs east of its default Output Portal: a lane of `count` pairs on the row below Push, starting directly under its anchor, as Orca's `P` writes (ADR 0070's amendment on Push's lane). `index` counts pairs, as Track's does. The lane is ordinary Source, not locked.

The first build selected the pairs east of Push's last operand, on its own row; the user then chose Orca's lane. Rework that selection and its tests; the ordering, missed-diagnostic and value-carrying work stands. It is ordered as ticket 04 decides. `@t` is free once ticket 09 has moved Track to `&t`. Push has no absolute form.

## Acceptance criteria

- [x] `@t` is in the Function table with Number operands `index` and `count` and an untyped `value` operand that carries its encoding unchanged, as a Write's does; an unwritten operand makes it invalid (ADR 0069).
- [x] A `count` of `00` diagnoses at the Turn, and a pair past the row edge or a lane below the Grid diagnoses and writes nothing, and a nested Push still returns `value`.
- [x] Push writes `value` into the pair `index % count` pairs east of the Cell directly below its anchor: `@t0003C4` writes Cells 0–1 of the row below, `@t0103C4` Cells 2–3, `@t0403C4` Cells 2–3 again; a nested `value` does not move the lane.
- [x] Tests drive same-Tick writes to `index`, `count` and `value`, and readers of the selected pair before and after it, as ticket 04 decides.
- [x] A Track whose list is Push's lane, standing on the row below west of the lane, reads within the Tick what Push writes when it does not feed the Push; when it does, it reads the write on the next Tick, and no cycle is diagnosed.
- [x] A Function standing in the lane is not locked: it takes its Turn, and Push's write over it suppresses or replaces it as ticket 04 decides.
- [x] Push answers `value` only at the selected pair; nested, it returns `value` to its parent.
- [x] A Source holding a Push gives the same result from a reused schedule as from a fresh one.
- [x] The Function reference shows Push writing into the lane below it. ADR 0019's table and the glossary already describe the lane.

## Resolution

Push selects its pair with its own `PairSelection::Lane`: `index % count` pairs east of its default Output Portal, on the row below, whatever its operands hold. Track keeps its east-of-last-operand rule, and the two share only the `index % count` that names its Function in a zero-count diagnosis. The ordering, missed-diagnostic and untyped-value work from the first build stands, and a `value` literal paints with the default plain Source text.
