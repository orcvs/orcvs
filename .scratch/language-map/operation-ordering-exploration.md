# Operation ordering: executable exploration

2026-09-06. Status: design direction accepted and recorded in [ADR 0032](../../docs/adr/0032-schedule-tick-execution-by-dependency.md); implementation tracked in [tick-execution-order](../tick-execution-order/spec.md). The observations below describe the exploration before production integration, not a completed scheduler.

Open [the self-contained walkthrough](operation-ordering-prototype.html). It uses actual Orcvs Function spellings in a deliberately small JavaScript model. Fixed Portal destinations are hypothetical configuration displayed alongside the Source, not proposed syntax. It does not run the Rust parser or MIDI backend.

## Question and candidate

Can one execution order deliver both current Note data and current Bang activation to MIDI during T, including when a producer is below its consumer?

The candidate is `parse → establish dependencies → topologically order → execute → publish`. A root waits for all possible suppliers to settle. A supplier can settle with a value, no output, or a diagnostic. Merely finishing a Bang-producing calculation does not activate anything. Each scheduled root executes at most once. Independent ready roots use row-major order as a deterministic tie-break.

The graph includes data dependencies and activation dependencies. Nested Functions remain inside their containing Expression. There is no executable standalone Bang Function and no preliminary run of every Bang-producing Function.

For C4 plus Bang, the required order is simply:

```text
Note producer ── Note operand ──┐
                               ├── MIDI Play, during T
Bang producer ── activation ────┘
```

Moving the Bang producer below MIDI changes the Portal destination, not the dependency direction. A fixed `Y-1` Portal therefore needs no repeated evaluation.

## What the current code permits

The existing `source::tick::plan(...) -> TickPlan` interface is the right seam. Playback can continue consuming ordered Play Commands and Source can continue committing an atomic outcome. Scheduling, operand resolution, and activation belong behind that interface.

The parser already records expected tokens for missing and invalid operands. However, `Expression::entries()` hides those records, the Language Map retains only complete runtime Atoms and a leading Function candidate, and parsing stops at the first invalid operand. A scheduler needs parser-owned positions and states for the complete operand layout, including nested ownership and expected inputs beyond an incomplete Source run. It must not reconstruct signatures or Bang eligibility from glyphs.

The shipped Source grammar currently reaches scalar value Functions and terminal Functions. Sequence machinery exists in the Evaluator, but Source-parseable Sequence constructors do not yet exist. This makes a first fixed-destination, two-Cell result scheduler a concrete bounded implementation rather than a general dynamic-dataflow project. Function definitions still need to describe possible output effects sufficiently to establish dependencies before calculation.

Relevant code:

- `lang/src/parser.rs`: permissive analysis, including the early return on an invalid operand.
- `lang/src/expression.rs`: complete, incomplete, and invalid records.
- `lang/src/atom.rs`: Function declarations and contextual literal conversion.
- `orcvs/src/source/language_map.rs`: parsed Expression ownership and valid Bang positions.
- `orcvs/src/source/portal.rs`: result destinations and whole-write admission.
- `orcvs/src/source/tick.rs`: the scheduling seam, current ordering, encoding, and result delivery.

## Preserve the meaning of spatial writes

Current spatial writes transport **encoded Cells**. Nested Expressions transport **typed values**. These are observably different:

```text
.^3C        evaluates to Note C4
.+C401      evaluates to Number C5 (197)
.+.^3C01    diagnoses: Add received Note C4 in a Number operand
```

Writing `C4` into a Number operand causes that operand to read hexadecimal C4, or 196. Directly injecting a Note Atom would change this behavior. A scheduler can preserve the existing rule by projecting the encoding and decoding it using the destination's already established operand type. This does not require reparsing the whole program after every write.

A temporary native probe against the built `lang` library confirmed those results, plus:

```text
.=0101      Bang
.=0102      Empty / no write
!>007F      incomplete, with expected Note token retained
!>00**C4    invalid Number operand; not a Bang activation source
```

The invalid example's token list ends at the invalid Number. This demonstrates why retaining existing `tokens()` alone cannot describe every later input slot.

## Bang lifetime: confirmed during exploration

The user confirmed that `**` is visual output and manually entering it is a no-op. The walkthrough implements that rule as **outputs-only pulses**: at the start of T, discard valid standalone Bangs left in the previous output display; during T, only fresh results activate neighboring roots; display those results until the next Tick begins. Invalid `**` in an operand is still invalid syntax and is not cleared as a pulse. The clearing mechanism is a prototype implementation choice; the visual-output/no-op distinction is the confirmed domain rule.

This has no generation metadata and does not replay a pulse on T+1. It also deliberately means a manually entered standalone `**` does not activate a Function. This is the confirmed Orcvs authoring rule. Original Orca's raw glyph behavior is not uniformly a manual no-op; the earlier upstream probes remain authoritative about that implementation.

The alternative of making manually entered Bangs trigger would require a separate display overlay or explicit pulse provenance in execution state. Identical Source bytes alone cannot distinguish an old generated `**` from a newly typed `**`. The confirmed no-op rule removes that requirement.

## Limits that remain deliberate

The prototype rejects competing writers and instantaneous dependency cycles. That establishes a clear executable subset; it does not establish permanent language restrictions. False predicates still settle, so their dependents do not wait forever.

Computed destinations and variable encoded widths need footprint resolution before consumers can be scheduled. Source edits that replace Function identities or Expression structure cannot change a precomputed graph unnoticed. A bounded implementation can defer structural writes or diagnose unsupported projections; those alternatives are not equivalent to the existing fallback's suppression of overwritten turns.

Pulse activation must use parser ownership at the destination. A possible `**` landing in a Number input is an invalid projected operand, not a new neighboring pulse. Whole-slot projection is a useful initial restriction; partial writes and writes that join formerly separate runs need further language rules.

The walkthrough retains today's behavior in which value roots run each Tick and MIDI requires activation. The broader rule for which Function families are intrinsically active remains separate from the ordering experiment.

## Recommended next production step

Preserve parser-owned operand layouts first. Then replace the internal row-major traversal with a scheduler for fixed scalar projections, keeping Cell encoding semantics and the external Tick interface. Require evidence for both producer layouts, a multi-stage data chain, missing input completion, no-Bang completion, invalid operand nonactivation, conflicts, cycles, and pulse lifetime before extending the supported projection shapes.

The architectural result is narrower than “build a new VM”: retain the existing Expression Evaluator; make Source execution own a reliable schedule. The remaining decisions concern the language's observation and lifetime rules, rather than the mechanics of topological sorting.

## Exploration evidence

Nine guided scenarios were smoke-executed from the HTML's exported `OrcvsOrdering` module using Node's `vm`: above, below, calculation chain, false comparison, competing writers, cycle, no replay, invalid embedded Bang, and manual Bang. Both script blocks compiled. The page's reset, plan, execute, next-Tick, and edit handlers were also exercised with a DOM shim. No automated test suite was added for this throwaway model.

The exact upward case places the Bang producer at `(0,4)`, its Portal at `(0,3)` (`Y-1`), and MIDI at `(0,2)`. MIDI emits C4 during T. Changing equality to false for T+1 gives no MIDI output, no diagnostic, and blank Cells where the old pulse was displayed. This was rerun after correcting the scenario to use exactly `Y-1`.

`git diff --check` passed. Browser visual verification was not performed. No production Rust was edited during this exploration, so the earlier fallback's Rust gate evidence is unchanged; its implementation does not yet implement the confirmed lifetime rule or dependency scheduler.

The user accepted the scheduling direction. The prototype remains a local exploratory artifact; `tick-execution-order/03` records its capture on a throwaway branch alongside final integration. This JavaScript subset is not a production implementation.
