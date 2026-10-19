# A committed Tick records its Portal accesses

Status: accepted. Leaves [ADR 0009](0009-portals-resolve-tick-plan-destinations.md)'s rule that a Portal is internal Tick state, never a language value or persistent state, unchanged: what survives the Tick is a record of what its Turns did, not a Portal. Settled in `.scratch/tick-portals/`.

Orca lights each operator's ports as it runs, so its tracker shows a read head moving down each voice's column. Orcvs resolves a dynamic Input Portal at the Turn and drops it, so nothing outside the Tick knows which Cells a Function read. The console can paint only what a Render Frame can derive, and a Source revision alone does not say which Cells the Tick before it read. The Tick has to say.

## Decision

**A Tick plans its Tick Portals with its Tick Plan.** Tick Portals is an immutable, ordered list of Portal accesses, one entry per access, each carrying the Tick it observed. Execution is its only writer, and recording is unconditional: there is no generic recorder, no observer a caller installs, and no switch that turns recording off, so the record a test reads is the record Playback commits. The Tick Plan and the Playback Engine are unchanged; the record travels beside the plan, not in it.

**Each entry names its owner as execution held it.** The owner is the anchor of the Function whose Turn made the access and the Function that Turn executed, after any Function Replacement the Tick made before it. Both are captured during execution from the schedule's computation and the Turn's running Function, never recovered from the committed Source, which the Tick may have rewritten.

**A read is recorded once the Turn passes the writer wait.** It is the Input Portal the Interpreter is handed, with its origin — `Declared` where the Function declares the Portal at a fixed offset, `Selected` where its operands chose it this Turn — and its site. The site is the Input Span it resolved inside the Grid, as many Cells as the Turn reads, or the Portal Error that says why none resolved. A resolved site says a site was read, not that it held a value: whether the operand bound is the Turn's diagnostic, not the record's. A Turn that waits for a writer records nothing until it is retaken, so a retried Turn leaves exactly one read. A Turn the schedule refuses before interpretation — syntax, arity, suppression, an operand that does not decode, a Rest — records nothing, and its diagnostic stands as before.

**Order is execution's.** Turns appear in the order they settled, and within a Turn its read comes first. Writes, Halt locks and Source Bundle parts belong to the same record, after the read in the order they ran, each recorded where the decision that admits or refuses it is made; `.scratch/tick-portals/` tickets 03 and 04 add them.

**An Input Span is minted only from the Input Portal a Turn resolved.** Its constructor is private to Tick execution and it exposes only a read-only Span, so no Span from anywhere else can stand for a read.

**The record is committed through the one commit path.** The optimistic attempt carries the record in its planned Tick, and the locked attempt plans it under the guard; both hand plan and record to the same Source commit, which stores the record with the revision that commit leaves. A refused attempt is dropped whole and publishes neither. Every committed Tick replaces the record, so a Tick that accessed no Portal leaves an empty record rather than the last one.

**A Source revision answers the record committed with it.** A revision read while the Source is still at the revision a Tick's commit left carries that record; a revision an edit has made since carries none, so a record is never painted against Cells it was not committed with. A new or loaded Source has none until its first Tick, and the record is not persisted.

**The Render Frame paints resolved reads as Input Portal.** Derivation sets the Input Portal role on each Cell a resolved read covers, in place over the Cells it has built, so a record costs no allocation per frame. A read whose site did not resolve stays in the record and paints nothing.

## Consequences

- The console lights each `&v` read of the tracker as it steps, with the Input Portal role ADR 0009's static Output Portal highlight sits beside.
- `TickPortals`, `PortalUse`, `PortalEvent`, `PortalOrigin`, `InputSpan` and `PortalError` join the public surface of the `orcvs` crate, which no one outside this workspace consumes.
- Every Tick allocates its record, including a Tick that reads nothing, which allocates the empty record's handle and no entries.

## Considered options

- **A recorder the caller installs.** Execution would call out to an observer at each access. It splits the record a test reads from the one Playback commits, and a shipped seam a caller can leave empty is a branch only a test needs.
- **Folding the record into the Tick Plan.** The Tick Plan is what the Playback Engine applies; the record is what the console observes. Keeping them apart leaves the engine's input unchanged.
- **Deriving the paint from the committed Source.** A Tick may rewrite the Cells its owners stood in, and a dynamic read's site depends on operands the Tick itself wrote. Only execution knows which Cells were read.
