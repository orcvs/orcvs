# Order Tick effects by Source Position

Timing, ordering, and Bang-lifetime clauses below are superseded where they conflict with [ADR 0032](0032-schedule-tick-execution-by-dependency.md). Other decisions remain in force.

At the beginning of each executed Tick, the Playback Engine dispatches scheduled expiries due for that Tick before any new Play Commands from its Tick Plan. Those expiries are time-owned playback work rather than Language Units in the new Source Snapshot, so they do not inherit the original Play Function's Source Position.

Tick planning identifies candidate roots from the starting Source Snapshot and orders them by the dependencies established under [ADR 0032](0032-schedule-tick-execution-by-dependency.md). Row-major Position breaks ties between ready roots. Every Expression root has one producer Position—its anchor—and effects from one producer retain emission order.

Only original Function candidates receive scheduled evaluations. Generated Function spellings gain no evaluation in the same Tick. Generated operand values and fresh Bang results are visible to dependent original roots. A Source-resident `**` is display state and causes no activation; it is cleared before evaluation when it remains a valid parsed Bang.

Each complete Atom or Sequence write validates its whole destination before it enters the Tick Plan. An invalid or out-of-Grid destination diagnoses and contributes no partial write. After validation, conflict resolution is Cell-wise: a later producer can overwrite part of an earlier producer's encoding, and the resulting Source may intentionally contain an alignment or syntax diagnostic, visible to subsequent turns and after commit.

Within one successful Self-Banging Function move, spaces written over the old Span precede the Function spelling written at the shifted destination, so the later write wins where horizontal movement overlaps one Cell. A blocked move instead replaces the current Span with `**`. Multiple effects or terminal commands from one producer retain emission order.
