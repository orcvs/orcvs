# Music and dataflow execution order: evidence for Orcvs

Researched 2026-09-06 against primary documentation. This note is design research, not an accepted language decision. The working-Source implementation and ADR 0031 are a provisional ordered fallback while the broader `parse → establish order → execute → output` direction is considered.

## Verified behavior in other systems

### SuperCollider: establish producer-before-consumer order

SuperCollider's server evaluates synth nodes in a defined order during each control cycle. A synth consuming another synth's audio output must follow its producer. Users establish that order with node placement, movement, and groups. This is server execution order, distinct from client-language statement order. The documentation also distinguishes current-cycle audio reads from control reads that may observe older data; `InFeedback` permits a feedback read with up to one block of delay. This is not automatic general dependency scheduling. [Official order-of-execution guide](https://doc.sccode.org/Guides/Order-of-execution.html)

**Orcvs inference:** separate the order in which Source is presented from the order in which dependent calculations execute. Same-Tick note generation works when its producer precedes the MIDI consumer in the execution schedule, regardless of screen coordinates. A retained previous value is not a substitute for the current Tick's value unless the language explicitly requests feedback.

### Faust: feedback has temporal meaning

Faust represents signal processing as composed block diagrams. Its recursive composition `A ~ B` inserts an implicit one-sample delay on the connections from A to B; its explicit one-sample delay primitives include `mem`, apostrophe, and `@(1)`. Consequently, this recursive form describes a recurrence using previous-sample state, not a request to repeatedly evaluate a same-sample loop until it settles. Faust's samples are not Orcvs musical Ticks; copying that time unit would be an additional design decision. [Official Faust syntax and delay documentation](https://faustdoc.grame.fr/manual/syntax/)

**Orcvs inference:** distinguish “write upward” from “feed back in time.” An upward edge can be an ordinary same-Tick dependency. A cycle needs an explicit Tick delay or a diagnostic, rather than an arbitrary traversal direction or generic reevaluation loop.

### Max: values and triggers need a deliberate order

Max distinguishes hot inlets, which trigger output, from cold inlets, which update state without triggering output. [Official inlet documentation](https://docs.cycling74.com/userguide/objects/)

Its `trigger` operation emits outlets from right to left with declared output types. [Official trigger reference](https://docs.cycling74.com/legacy/max8/refpages/trigger)

Cycling '74's example uses `t b f` to send a value to a cold inlet before sending a Bang to the hot inlet. The same article demonstrates that explicit trigger ordering avoids a patch changing behavior merely because connected boxes move on screen. [Official trigger tutorial](https://cycling74.com/articles/my-favorite-object-trigger)

**Orcvs inference:** the MIDI consumer must wait for the current note data as well as the current activation decision. Merely processing Bang-producing Functions first is insufficient. Unlike Max, Orcvs could infer this order from resolved typed operand destinations and Bang adjacency, rather than asking the musician to add trigger wiring.

Pure Data's manual endpoints were unavailable through the research browser (403/safe-open failures), so this note makes no independent claims about Pure Data. Max's first-party sources provide the relevant verified hot/cold and trigger behavior.

## Candidate: schedule the dependency graph, then execute once

The candidate below is an Orcvs design inference, not a claim that any cited system already implements Orcvs semantics.

```text
Parse typed Expressions and potential output geometry
                      ↓
Establish producer → consumer dependencies
                      ↓
Reject instantaneous cycles; schedule remaining roots
                      ↓
Execute each scheduled root at most once with current inputs
                      ↓
Commit visible Source output and deliver ordered MIDI for this Tick
```

For the acceptance example, the graph is simply:

```text
Note producer ── current Note operand ──┐
                                      ├── MIDI Play ── C4 during T
Bang producer ── current activation ───┘
```

Either producer can be above or below MIDI. `Y-1` changes an edge's destination, not whether that edge points into the past. No root has run “too early,” because the schedule is established before executing it.

Dependencies should represent potential writes, not only writes whose value is already known. Equality might return Bang or no value; MIDI still waits until that producer has decided. It then executes once if activated, using the resolved current operands. Nested value Functions remain part of their containing Expression; they do not each need a separate spatial turn.

This is stronger than a value-first/terminal-last split. A chain of value producers also receives the correct dependency order, and the same mechanism accounts for an activation prerequisite on an ordinary Function if that language rule is adopted.

## Conditions that make it bounded and understandable

1. **Know destinations before executing dependent roots.** Ordinary below-Source output and a fixed upward Portal are straightforward to resolve from parsed geometry. A runtime-computed Portal can work if its address is resolved from a separate acyclic calculation first. Fully unrestricted addresses require conservative potential destinations or a different contract; pretending they are statically known would be unsound.
2. **Represent absent output explicitly during scheduling.** A producer can finish without emitting Bang. Dependents must become ready after that decision; they must not wait forever for a pulse.
3. **Settle competing writers.** Two producers writing the same operand need a stated winner or a diagnostic. Row-major tie-breaking among otherwise independent roots can preserve a stable preference, but it cannot universally preserve old row-major last-writer semantics while dependencies reorder those writers. A contested Cell may need its own resolved-write step before any reader proceeds.
4. **Separate current data from generated code.** Same-Tick values should flow through established operand destinations. Replacing Function spellings should affect the following Tick's parsed graph. Otherwise, the graph being executed can invalidate its own topology.
5. **Require delayed feedback.** After removing explicitly delayed dependencies, a cycle should diagnose before any affected MIDI command is delivered. A delay reads retained Source state for T and records its replacement for T+1. Do not silently insert a delay according to geometry.
6. **Give a generated Bang one temporal identity.** Its `**` display must not independently trigger another event next Tick merely because the glyph remains visible. Whether a manually entered `**` denotes an initial pulse is a separate authoring rule. Reusing the glyph as both display and event state requires an explicit lifetime contract.

A standard acyclic graph can be ordered without executing roots repeatedly; implementing that graph is not inherently a fixed-point engine. The difficult work is defining accurate dependencies where Source writes can alter parsing, destinations, and competing writers.

## Architecture recommendation

Explore the scheduling candidate before treating the ordered fallback as settled. Keep scheduling inside the Source execution module: its interface should give callers one Tick outcome, with dependency analysis, execution, and output consistency behind that seam. That concentrates knowledge and verification in one place, improving locality and giving playback callers leverage. No adapter is justified merely to create this internal seam.

The first useful prototype should use only existing typed value Functions, terminal output, fixed Portal destinations, and explicit or diagnosed cycles. It should demonstrate the same note/Bang/MIDI patch with both producer layouts, a multi-stage producer chain, no-Bang output, a competing writer, and a feedback cycle. This tests the semantic model rather than benchmarking or prematurely generalizing the scheduler.

The sources establish two useful principles: musical systems make order explicit, and delayed feedback is different from current computation. They do not prove that a general self-modifying Source language has a statically resolvable graph. That is the design question the prototype must answer.
