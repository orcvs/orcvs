# Dependency ordering for one musical Tick

Research date: 2026-09-06. Status: research and a constrained design candidate, not an accepted execution contract. The working row-major implementation is a provisional fallback. This note supersedes the earlier suggestion that an upward Bang inherently needs an impractically complicated scheduler.

The user's proposed pipeline is sound as a direction:

```text
Parse → establish dependencies and execution order → execute → publish output
```

An output Portal at `Y-1` is not itself difficult. If its destination and the receiving Function are known before execution, the scheduler can put the producer first regardless of their Positions. The substantive decisions are how Source establishes dependencies, which version of a Cell a reader observes, and how mutations can change the parsed program.

## What the primary sources establish

### 1. LLVM: dependency graphs include more than producer-to-consumer arrows

**Verified:** LLVM's Data Dependence Graph models relations between instructions; its Program Dependence Graph also represents control dependencies. Memory edges cover flow, anti, output, and input dependencies. Cycles are represented explicitly and restrict reordering. [LLVM, Dependence Graphs](https://llvm.org/docs/DependenceGraphs/index.html)

**Application / inference:** Orcvs needs to describe at least these relations:

| Relation | Orcvs example | Scheduling meaning |
| --- | --- | --- |
| Data / flow | Note producer writes MIDI's Note operand | Producer completes before MIDI reads the new Note |
| Activation | Equality might produce Bang beside MIDI | MIDI waits for the pulse decision, then executes only if active |
| Write conflict / output | Two Portals overwrite the same operand Cells | A rule chooses the winning value; ordering cannot decide this accidentally |
| Read-before-write / anti | A Function intentionally uses the old contents before another replaces them | Preserve that version or explicitly change the language's meaning |

A compiler normally preserves an already specified program's behavior. Orcvs is deciding that behavior. Importing a dependency algorithm cannot answer whether an old Note or this Tick's newly produced Note was intended.

### 2. LLVM: destinations need extents and uncertainty must remain explicit

**Verified:** LLVM represents memory locations with both a starting address and a size. It distinguishes disjoint, overlapping, guaranteed-alias, and possibly-aliasing locations. Its read/write summaries are conservative when access is uncertain. [LLVM, Alias Analysis](https://llvm.org/docs/AliasAnalysis.html#representation-of-pointers)

**Application / inference:** a Portal's anchor alone cannot establish all Orcvs dependencies. A two-Cell Note and a variable-length Sequence from the same Portal cover different operands or roots. Unknown destination or width must mean “might affect this reader,” a diagnosed unsupported case, or a deliberate delayed write—not “independent.” Conservatively connecting every possible writer is correct only under an explicit observation contract and may make otherwise useful programs cyclic.

### 3. Ptolemy: establish ordering before execution, with explicit feedback delays

**Verified:** Ptolemy's synchronous dataflow domain determines firing order before execution. Constant input/output token counts enable this scheduling. Data-dependent firing patterns are outside that SDF model. Feedback loops require delays, represented by initial tokens, and an undelayed loop is rejected. [Ptolemy SDF introduction and delays](https://ptolemy.berkeley.edu/ptolemyclassic/almagest/docs/user/html/sdf.doc1.html)

**Application / inference:** Orcvs could adopt a finite dependency graph per Tick while keeping musical time outside it. A dependency chain completes within T; only explicitly delayed feedback reads T-1. Bang's optional presence needs an explicit settled outcome—Bang or no Bang—so absence does not leave a consumer waiting indefinitely. This is an adaptation, not a claim that Orcvs is SDF: optional activation and variable Sequence output violate assumptions of the simple SDF model.

### 4. TensorFlow: ordering can be explicit without passing an ordinary operand

**Verified:** `Graph.control_dependencies` orders dependent operations after specified operations complete. Multiple dependencies combine. This mechanism is separate from ordinary tensor operands. [TensorFlow Graph control dependencies](https://www.tensorflow.org/api_docs/python/tf/Graph#control_dependencies)

**Application / inference:** a Bang can remain a typed result while its presence is an activation dependency of MIDI. MIDI does not need a new Number/Note operand to express “wait for the pulse calculation.” Completion of a pulse producer is distinct from the producer actually returning Bang. Neither this source nor the LLVM source implies that completion should unconditionally activate an Orcvs root.

## A viable constrained model to explore

Everything in this section is a proposed Orcvs rule, not a fact about current execution or a recommendation already accepted by the user.

### Parse establishes a stable program for T

The Language Map identifies existing root Functions, nested Expression structure, typed operand slots, valid literal Bangs, and output destinations. Missing data can be represented as a typed slot if a producer supplies it. Invalid Bang-as-Number syntax still diagnoses; it must not become a free pulse.

During this Tick, data can change without changing this structure. Writing a Function, separator, comment, or other structural Source content affects parsing at T+1. That restriction preserves the possibility of “parse once, order once.” Allowing those writes to redefine pending Expressions in T instead requires invalidating or rebuilding part of the schedule.

The existing parser does not yet expose a complete slot-and-link representation. In the current checkout, `lang/src/interpreter.rs` accepts an `Atoms` list and explicit Tick inputs; it does not read a grid. `orcvs/src/source/language_map.rs` retains parsed Expressions and candidate roots. `orcvs/src/source/portal.rs` admits a destination using the already encoded result. Adapting these modules is work, but their existing separation does not prohibit dependency scheduling.

### Ordering establishes who supplies each input

For an initial prototype, accept only destinations known before execution, writes with known extents, and at most one producer per destination slot. Report ambiguity rather than selecting a winner based on whichever node happens to be ready first. This is a scoped way to test the model, not a proposed permanent ban on overlapping writes.

Create data dependencies for typed operand inputs and activation dependencies for every potential Bang output beside a root. Literal Bang values can supply an initial pulse directly. Equality producers contribute a dependency even when this Tick's answer will be no Bang: the scheduler cannot know the outcome before evaluation.

Sort the acyclic graph. Row-major Position can be a deterministic tie-break for independent ready roots. It should not override a dependency. Nested Functions can remain internal to one Expression's Evaluator unless they independently read or write Source.

### Execute each scheduled root once

A root receives this Tick's settled operand values and activation facts. An inactive root settles without producing effects. A failed producer settles with an error; its consumers need a defined error policy rather than silently reading stale data. A Bang is a result, never an executable `**` Function.

Results flow within the evaluation. Their displayed encodings can still accumulate for Source commit. Keeping external Source publication atomic does not require frozen input values inside the Tick.

### Output publishes an already resolved musical result

After evaluation, publish Source and the MIDI commands belonging to T. Pulse display and activation lifetime must be specified independently: rendering `**` after commit must not accidentally replay the same event on T+1. Preserve an explicit deterministic order for multiple MIDI commands, particularly commands sharing channel and note.

## The C4 and upward-Bang acceptance case

Suppose producer N supplies MIDI root M's Note slot and producer B writes Bang next to M through a Portal resolved to `Y-1`.

```text
N: Note C4 ── data ────────┐
                         ├── M: MIDI Play ── output during T
B: Bang ─── activation ────┘
```

The graph requires N before M and B before M. B's physical location below M is irrelevant. N and B can run in either order if independent. M runs once with C4 and an active pulse, during T. No separate evaluation of all Bangs is needed, and a producer's nested calculations run as part of that producer.

This remains straightforward if B's destination is a fixed relative offset. If calculating the destination requires N's output, destination resolution itself has a dependency and must occur before identifying B's consumers. It can be staged when those dependencies are acyclic and isolated; it cannot always be folded into a single order-before-execute phase.

## What still requires explicit decisions

- **Feedback:** N depends on M and M depends on N. Reject a same-Tick cycle, or require an explicit delayed connection with initial data. An existing value in Source does not by itself declare which dependency should be delayed.
- **Multiple writes:** reject conflicting producers, define priority, or introduce a merge rule. If choosing priority, specify whether losing writers still execute and what consumers see when the highest-priority producer emits nothing or fails.
- **Conditional output:** does no new Note preserve the old Note? Does no new Bang mean inactive even when last Tick's `**` remains visible? These answers must be distinct if pulses are transient but notes persist.
- **Dynamic Portal:** fixed relative `Y-1` is easy; computed coordinates and computed widths are a separate category. Decide what can be resolved from initial Source and what may depend on this Tick's data.
- **Sequences:** transporting one typed Sequence over one logical connection keeps topology stable despite a changing element count. Spreading its encoding over arbitrary Source Cells makes the write extent and downstream topology dynamic. These are different execution contracts.
- **Program mutation:** stable parsed structure supports this pipeline. Same-Tick rewriting of Function identity or Expression extents requires a more dynamic model.
- **Pulse lifetime:** a displayed Bang must have exactly one defined activation interval; returning a persistent display value must not silently generate a second musical event.
- **Diagnostics:** unrelated valid roots should be able to continue when a graph region is invalid, if that is the desired live-editing behavior. The dependency graph can identify affected consumers; the scope of suppression is a language decision.

## Discussion recommendation

Prototype the constrained typed graph before deciding that backward Portals are too complex. Use fixed destinations, unambiguous typed inputs, explicit no-Bang outcomes, and a diagnosed cycle. Exercise same-Tick C4 plus Bang with the producers above and below MIDI, an absent pulse, a missing Note filled by a producer, a write conflict, and a feedback cycle.

This module would gain depth by owning input resolution and scheduling behind the Tick interface. The leverage is one timing contract for Notes, Bangs, and other results; the locality is that callers and tests no longer reconcile separate scheduling conventions. No new adapter seam is justified: this is in-process language execution.

The ordered working Source remains a concrete fallback for unrestricted Source mutation. It should not be presented as the final design while this alternative is under discussion.
