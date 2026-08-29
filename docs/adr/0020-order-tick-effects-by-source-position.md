# Order Tick effects by Source Position

Tick planning follows Orca's single row-major pass over the Language Map derived from the Source Snapshot. Every actionable Language Unit and Expression root has one producer Position—its anchor—and all Cell effects, activation deliveries, locks, diagnostics, and terminal commands are ordered first by that producer's row-major Position and then by the order the producer emits them; later Cell effects win conflicts at each Cell independently.

Only units present in the Source Snapshot receive turns. A Source-resident Bang is visible to cardinally adjacent roots at their turns and plans its own expiry from its anchor. An Activation Character plans movement from its anchor. An Expression uses its root anchor, and a Jump chain uses its head anchor. Planned writes never gain a turn in the same Tick.

Each complete Atom or Sequence write validates its whole destination before it enters the Tick Plan. An invalid or out-of-Grid destination diagnoses and contributes no partial write. After validation, conflict resolution is Cell-wise: a later producer can overwrite part of an earlier producer's encoding, and the resulting Source may intentionally contain an alignment or syntax diagnostic on the next Tick.

Within one successful Activation Character move, clears of the old Footprint precede writes of the new Footprint, so the new write wins where horizontal movement overlaps one current Cell. A blocked move instead replaces the current Footprint with `**`. A Source-resident Bang expiry clears its current Footprint. Multiple effects or terminal commands from one producer retain emission order.
