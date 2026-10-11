# A standalone Bang is a Function, as Orca's `*` is an operator

Status: proposed. On acceptance, supersedes [ADR 0071](0071-a-bang-travels-as-its-glyph.md)'s third decision and the consequences that follow from it, [ADR 0032](0032-schedule-tick-execution-by-dependency.md)'s clause that a Bang is not a standalone Function (its value half stands), [ADR 0006](0006-bang-activation-includes-self-banging-functions.md)'s contact activation and its clause that a Halt over an occupied non-root diagnoses, as it applies to `**`, and [ADR 0067](0067-track-reads-a-portal-as-jump-does.md)'s restriction of the missed-Bang diagnostic to dynamic writes; restores [ADR 0003](0003-source-snapshot-is-complete-language-state.md). Settled in `.scratch/bang-function/issues/07-settle-clearing-and-halt-precedence.md`.

ADR 0071 told a typed `**` from a written one by having Source remember which Cells the previous Tick wrote as Bang display. That record broke ADR 0003: two identical Source Snapshots at the same absolute Tick could plan differently, a reloaded Source fired a Bang the live one would not, a lone `*` typed beside a written `**` reparsed one Cell west and fired a Bang nobody typed, and a Bang written into an operand was forgotten after one quiet Tick.

## Decision

**A standalone `**` in Source is the Bang Function, and follows the rules every Function follows.** It has no operands, is root-only and intrinsically active, and can emit Bang. Its Turn does what every standalone `**` already did before the first Turn: it activates the roots aligned with its Span and clears its two Cells. Only its place moves, from before the schedule into it. Like a Self-Banging Function, it reserves the Span it clears, and that reservation is not a self-dependency. The parser's existing root-only `**` arm names it; operand parsing keeps reading `**` as the Bang value. A `**` in an untyped operand stays the Bang value a pass-through carries; in a typed operand it stays invalid syntax. A Function that answers Bang writes `**` and activates its aligned roots in its own Turn, as before; the `**` it leaves is Source, and a Bang Function on the next Tick. Nothing about Bangs is remembered between Ticks.

**An admitted overwrite before its Turn suppresses it.** The ordinary writer-before-covered order puts a producer's write ahead of the Bang Function it covers, so a producer that bangs again or answers the Absence Marker suppresses its own previous `**`, as Orca's `D` clears its own `*`. A partial overwrite suppresses the whole Bang Function. A reservation or a refused write suppresses nothing.

**A clear is the producer's ordinary Absence write, made in its Turn.** A producer that takes no Turn — Halt-locked, inactive, nested in a root that takes no Turn — writes nothing, and its last `**` fires once more at its own Turn. A producer that is invalid writes nothing, as ADR 0069 states. A producer stopped by a cycle writes nothing, and its `**`, a consumer of the stopped writer, stands under ADR 0068, diagnosed as waiting on the cycle. The same surviving-Bang rule covers a removed writer and a writer whose destination changed.

**Halt is an ordinary Producer.** Its lock is an Effect of its Turn and holds only if that Turn precedes its target's. Turns are ordered by potential supply (ADR 0032): a Halt waits for every Bang that could reach it, except a Bang from inside the subtree it locks. Only a Bang that executed activates it. A Halt directly above the `**` that would activate it therefore takes its Turn inert; the `**` fires, and its Bang reaching the Halt is diagnosed as a missed Bang. A Halt whose activator depends on its target is a same-Tick dependency cycle. A Halt activated independently first locks the `**`, which stands silent, Tick after Tick, until it is not locked.

**Placement and reading follow existing order.** A mover or emission reaching a Bang Function's Cells has no edge to it beyond the existing ones, so Source order decides under ADR 0060's rule 3b, as for a mover train: a placer later in Source order enters the Cells the Bang vacated, an earlier one finds them occupied. A reader of its Cells waits for it, as for any writer, and sees them cleared, still `**` if it was locked, or the overwrite that suppressed it.

**A blocked Self-Banging Function writes `**` and activates nothing in that Turn.** Contact activation is removed; the `**` is a Bang Function on the next Tick and activates every aligned root then.

**Every Bang delivered to a root that has taken its Turn is diagnosed as missed.** The diagnostic loses its restriction to dynamic writes.

## Departures from Orca

Orca's `*` erases itself at its own turn, and neighbours detect the glyph at theirs, so Orca's reach and timing follow reading order. Orcvs delivers activation when the Bang Function executes:

- A typed Bang activates aligned roots on every side, where Orca's later south and east neighbours miss the erased glyph. This already ships.
- A blocked mover's collision activates every aligned root on the next Tick, where Orca's south neighbour runs in the collision frame and the others the next.
- A Halt above its activating Bang misses it, where Orca's `H` locks the `*` first.
- A locked Bang delivers no activation while locked, where Orca's standing glyph is detected every frame.

## Consequences

Each is an existing rule applied to the new root; none has an exception.

- Whether a `**` fires depends on the Source Snapshot and absolute Tick only; save, load, paste and session restore need no Bang handling.
- Deleting a writer, halting it, invalidating it, or moving its destination in the Tick after it banged lets its `**` fire once more. No destination history prevents it. A Copy with partial input writes nothing, so the same holds.
- A typed `**` in the Output Portal of any intrinsically active writer is suppressed by that writer's output, so typing `**` beside a root fires it only where no writer covers it.
- A Halt over a `**` locks it. A locked `**` under a Copy is carried, and triggers, every Tick.
- A `**` in an Increment's or Interpolation's own Output Portal is a same-Tick cycle, as a Function writing what it reads is.
- A Copy carrying a Function spelling onto a standing `**` is refused as a Function replacement.
- A partial overwrite suppresses the whole `**`; the stray `*` left is not a Bang.
- A cycle-stopped `**` stands and is diagnosed as waiting on the cycle.
- A Halt whose activator depends on its target is a same-Tick dependency cycle, while a Halt directly above its activating `**` misses it: `lock_covers` reaches only producers inside the locked subtree.
- Collisions sound one Tick later, and collisions with a Comment or the Grid edge now sound. A Directional Bang Function whose mover is blocked on its first step is re-activated by that `**` and re-emits, alternating every Tick.
- A mover meets a `**` in Source order: one moving west or north enters after it fires, one moving east or south is blocked.
- An emission into a Bang Function's Cells relies on Turn-local emission occupancy (`.scratch/placement-semantics/issues/03-apply-turn-local-occupancy-to-emissions.md`); until then an emitter aligned with a `**` it would enter forms a cycle with it.

## Considered options

- **Keep the provenance record and fix its anchor and expiry.** Identical Snapshots would still behave differently, which is the defect.
- **Clear a halted or invalid producer's Output Portal.** Needs an Effect without a Turn and contradicts ADR 0069; the replay it prevents is the one accepted for a removed writer.
- **Order every placement after the Bang it enters, or every mover before it.** Either makes occupancy independent of direction, at the cost of an edge no other Function has, against ADR 0060's rule that occupancy alone creates no ordering edge.
- **Keep contact activation.** A blocked mover's `**` would then fire the contacted root a second time on the next Tick.
- **Clear the destination of a Copy with partial input.** Contradicts ADR 0069.
- **Extend the Halt's exemption to any activator that depends on its target.** A new transitive rule where the cycle policy already answers.
