# Execute against live typed expressions

Status: resolved

Tags: release/v1

## Problem Statement

A person composing on the Grid expects an earlier Function output to reach a later Function during
that Tick, including when the producer is below its consumer, changes only part of an operand, or
replaces a nested computation. The current bounded scheduler cannot deliver that behavior
consistently. It rejects partial and competing writes, excludes structural replacement, and stops a
spatial consumer when a supplier fails even if usable operand characters remain.

Those restrictions also conceal a distinction the composer needs: a nested computation returns a
typed value, while a spatial output supplies Source characters. The same Note spelling must remain
a Note when returned by nesting, but must follow the receiving literal signature when written
spatially. Decoding every intermediate write can report an error for characters that a later writer
repairs before the consumer executes.

The composer must also be able to understand the next Tick from the Source they see. Replacing a
Function must not silently recruit or delete parameters during execution, and rendering must not
erase untouched characters to make the next parse resemble the temporary execution tree.

## Solution

Use the Parser's positioned, typed expression structure for rendering and Tick execution. Establish
it from the starting Source, derive execution order from dependencies, and thread outputs through
its live inputs without running Source parsing between Function executions. Render the admitted
outcome into Source for the next Tick, whose interpretation uses the ordinary arity-based Parser.

Spatial writes update Pending Operand Encodings Cell by Cell. The Function's current signature
decodes each literal only when that Function executes. Nested results retain their types. One
computation may supply its parent and project its result through a Portal during the same execution.

Implement the bounded contract confirmed in ADR 0034 and assessed in the accepted integration
prototype. Preserve fixed destinations and existing activation behavior. Include nested ownership,
replacement, suppression, failure and publication from the start. Do not choose a storage layout or
claim a general scheduler for the deferred structural cases.

## User Stories

1. As a composer, I want a producer below its consumer to supply that consumer during the same
   Tick, so that Grid position does not delay my calculation.
2. As a composer, I want a chain of Functions to run in dependency order, so that each Function
   uses the results supplied during that Tick.
3. As a composer, I want independent Functions to use Position as their execution tie-break, so
   that overlapping outputs have a deterministic result.
4. As a composer, I want every potential writer to settle before its receiver executes, so that
   the receiver does not read an intermediate operand.
5. As a composer, I want one output to update Cells across two original parameters, so that
   spatial placement has the same meaning as editing those Cells.
6. As a composer, I want a later write to replace only the Cells it covers, so that uncovered
   parts of an earlier write survive.
7. As a composer, I want a Producer's multiple outputs to keep their emission order, so that
   overlapping outputs from one computation compose predictably.
8. As a composer, I want a whole spatial Note encoding in a Number operand to be interpreted as
   a Number literal, so that identical operand characters have identical meaning.
9. As a composer, I want a nested Note result to keep its type, so that arithmetic does not
   silently convert it into a Number.
10. As a composer, I want a spatially changed operand to remain pending until consumption, so that
    a temporary malformed encoding can be repaired within the Tick.
11. As a composer, I want an unrepaired malformed operand to diagnose when consumed, so that an
    invalid calculation produces no result.
12. As a composer, I want decoding to use the declared literal type, so that a malformed Note is
    not accepted merely because its characters could encode a Number.
13. As a composer, I want type and domain checks after decoding, so that syntactically valid
    characters outside a Function's domain still produce a clear diagnostic.
14. As a composer, I want a nested Function to compute once for both its parent and its Portal,
    so that the two delivery paths share one result.
15. As a composer, I want both consumers of that result to wait for its computation, so that
    neither delivery path observes stale data.
16. As a composer, I want a nested Portal to remain inactive with its parent, so that configuring
    an output does not cause an otherwise inactive Expression to run.
17. As a composer, I want a failed spatial supplier to leave existing operand characters intact,
    so that its failure does not erase usable Source.
18. As a composer, I want successful earlier writes to survive a later supplier's failure, so
    that a receiver can use the encoding actually left by the Tick.
19. As a composer, I want a parent to fail when a nested child supplies no typed result, so that
    missing nested computation is not replaced by unrelated Source characters.
20. As a composer, I want a child's successful spatial output to survive its parent's failure,
    so that a later parent error does not undo an already successful computation.
21. As a composer, I want every spatial destination to be validated in full before writing, so
    that an out-of-Grid output does not leave half a value behind.
22. As a composer, I want a rejected spatial destination to preserve the successful nested
    result, so that the parent can still use its child's answer.
23. As a composer, I want a dependency cycle to reject all execution effects for that Tick, so
    that I never receive a partial outcome from a program with no valid order.
24. As a composer, I want self-dependency to diagnose without re-execution or silent delay, so
    that feedback does not acquire an implicit timing rule.
25. As a composer, I want a value written over a nested Function to replace that computation as
    an input, so that the parent consumes the newly supplied operand during the same Tick.
26. As a composer, I want replacement to suppress the former computation and its descendants,
    so that detached Functions cannot emit through their Portals.
27. As a composer, I want a value written over a top-level Function to suppress its turn, so
    that the former Function produces no further output.
28. As a composer, I want untouched former parameter characters to remain in Source, so that
    replacement changes only the Cells the output covers.
29. As a composer, I want Function replacement at an original anchor to retain the original
    parameters and nested connections, so that replacement does not reshape an Expression to fit.
30. As a composer, I want an incompatible replacement arity or input type to diagnose, so that
    an invalid replacement cannot silently drop or acquire inputs.
31. As a composer, I want the replacement's signature to interpret retained literal characters,
    so that literal meaning follows the Function that actually executes.
32. As a composer, I want retained nested results to stay typed after Function replacement, so
    that changing a signature does not turn a returned value into character transport.
33. As a composer, I want a misaligned write to remain visible even if it discards a Function,
    so that preserving the old parse does not overrule Grid placement.
34. As a composer, I want newly anchored Function spellings to wait for the next Source parse
    within this bounded implementation, so that they do not gain an unplanned current-Tick turn.
35. As a composer, I want the next Tick to parse all resulting Source by arity, so that former
    parameters and newly exposed roots receive their ordinary interpretation.
36. As a composer, I want next-parse diagnostics to remain distinct from current execution
    diagnostics, so that a successful calculation is not confused with diagnostic-free future Source.
37. As a composer, I want existing Bang activation and display lifetime to remain correct, so
    that extending value propagation does not replay old pulses or sound a root twice.
38. As a composer, I want live edits and rendering to derive meaning from the same Parser
    output used by execution, so that the console and Tick planner agree about my Source.
39. As a maintainer, I want positioned parameters and ownership to come from the Parser, so that
    downstream consumers do not reconstruct a second interpretation of Source.
40. As a maintainer, I want the same Source Snapshot, Tick and fixed configuration to produce
    the same Tick Plan, so that behavior is reproducible without hidden execution state.
41. As a maintainer, I want acceptance checks at the Source/Tick boundary, so that I can change
    internal storage and scheduling without rewriting tests that encode the implementation.
42. As a maintainer, I want the old failed-supplier regressions revised explicitly, so that
    historical tests do not silently restore the superseded policy.
43. As a maintainer, I want native, WASM and existing feature behavior preserved, so that this
    semantic change does not narrow the application's supported configurations.
44. As a maintainer, I want prototype evidence and deferred questions recorded separately, so
    that acceptance of the bounded model is not mistaken for a general structural language contract.

45. As a composer, I want a Portal configured on a Terminal Output Function to diagnose, so that
    an invalid output configuration is visible rather than silently ignored.

## Implementation Decisions

- **Parser authority:** the Parser establishes positioned Functions, literal inputs, nested
  ownership, Expressions and diagnostics from Source. The same semantic product serves rendering
  and Tick execution. A derived presentation or lookup view may exist; it must not reinterpret
  spellings or reconstruct parameter positions independently.
- **Tick lifetime:** establish the initial parse for a Source revision, execute against Tick-local
  live expression state, then render Source for the next revision. Reuse of a valid parse or
  incremental row rebuilding is allowed. No Source parsing occurs between executions. Pending
  encodings and typed intermediate results do not become persistent Source state.
- **Partitioning:** retain ADR 0033's arity-based row walk, nested claims, Comment boundary,
  ordinary empty Cells within claims, row confinement and one-Cell recovery for unrecognized
  spellings. This work does not reopen the incomplete-Function interview.
- **Ownership and identity:** scheduled entries refer to original computations in the live
  expression structure. Distinguish a Function's own spelling and directly owned literals from
  the containing Expression's extent. A parent's extent is not a second claim on every descendant
  Cell. Replacement must update input connections and execution eligibility together.
- **Ordering:** derive dependencies from fixed output destinations and nested value connections.
  A writer precedes every Function whose relevant spelling or inputs it affects. All potential
  writers settle before the receiver, including producers that fail or emit nothing. Position
  breaks ties between independent ready computations; emission order orders one Producer's writes.
  Do not accept an execution order supplied by a fixture.
- **Suppression:** account for structural replacement before affected descendants can execute.
  An inactive parent or replacement of a computation suppresses its descendants and their Portals.
  A Portal does not independently activate a nested Function. Function replacement retains its
  nested connections; value replacement removes the replaced computation from its parent's input.
- **Exactly once:** a live computation gets at most one execution. A value used by both a nested
  consumer and a spatial consumer is computed once. Reaching an already-executed Function with an
  output is an ordering defect; do not repair it with another turn or a delayed write.
- **Spatial delivery:** every admitted write updates the covered Cells of live literal encodings.
  Whole and partial coverage use the same contextual interpretation. Preserve pending characters
  without intermediate decoding or an inferred replacement type. At execution, decode with the
  current Function signature, then perform type and domain checks. Unchanged retained literals use
  the replacement signature too.
- **Nested delivery:** pass the returned typed value along the nested connection. Do not serialize
  it into Source to bind it to the parent. Preserve existing Function-specific rules for typed
  inputs, including Numeric Conversion's identity behavior; spatial literal interpretation does
  not change those rules.
- **Dual delivery:** return one typed value, then project it through configured Portals as part of
  that computation's execution. Both consumers depend on it. This does not require an instruction
  to answer both a language value and a terminal effect; preserve ADR 0028's answer distinction.
  A Portal configured on a Terminal Output Function is invalid configuration and must diagnose.
  Reject that Portal without serializing the terminal effect as a value or silently ignoring the
  configuration. Preserve ordinary terminal activation and effect behavior; configuration validation
  does not activate an inactive root.
- **Write admission and composition:** validate a write's entire destination before any covered
  Cell changes. A rejected write diagnoses and applies no fragment. Valid writes compose Cell-wise;
  later coverage wins and untouched Cells retain earlier content. A rejected write does not remove
  the successful typed answer delivered to a parent.
- **Evaluation failure:** a failed producer diagnoses and supplies no spatial write. Its spatial
  consumers proceed after it settles, using surviving characters, including successful earlier
  writes. Its nested consumer fails if no typed result was returned. A parent's failure does not
  undo its child's successful spatial writes. Do not confuse failed evaluation with a successful
  absence result under existing language rules.
- **Cycle failure:** a self-dependency or dependency cycle rejects publication of all execution
  effects for that Tick, including independent writes, activation effects and Play Commands.
  Report the cycle. Ordinary producer or consumer evaluation errors do not use this whole-Tick
  rejection policy.
- **Value replacement:** replacing a nested Function supplies a literal encoding at that input
  under the spatial rule and suppresses the former computation and descendants. Replacing a root
  suppresses that root's turn. Former parameter characters remain unless a write covers them.
- **Function replacement:** at an original anchor, retain parameter positions, count, nested
  connections and fixed Portal configuration. Execute the replacement against those inputs and
  diagnose incompatible arity, type or domain. Do not recruit new Cells, discard inputs, or keep
  the former signature's literal interpretation. Only replacements preserving activation
  requirements and output kind are covered.
- **Misaligned output:** allow Grid-positioned output to overwrite part of an original spelling
  and discard that Function. Preserve the resulting characters, even when malformed. Retention of
  parameters for original-anchor replacement does not authorize same-Tick execution at a new anchor.
- **Publication and rendering:** preserve atomic Tick publication through the existing Source/Tick
  interface. Resulting Source reflects admitted writes and all untouched original Cells. Returning a
  nested value alone does not replace its Source spelling or erase its parameters. The next parse
  can expose new roots and diagnose leftover standalone literals; it need not reproduce the entire
  temporary live tree. Playback continues to receive the completed Tick Plan and does not interpret
  expressions or pending encodings.
- **Bounded configuration:** use fixed destinations. Extend the existing internal destination
  mechanism to exercise nested Portals and ordered emissions. Supply typed Function values only
  through a test configuration for the deferred Function-producing operation. This creates no new
  Source operation or public authoring syntax. Other producers must compute their values normally.
- **Representation:** storage choice remains open. No cell-indexed classification array, linked
  allocation scheme, arena, identifier width or immutable journal representation is mandatory.
  Choose the smallest coherent implementation behind the existing boundaries. Do not transfer the
  prototype's small Function table or conservative graph as a general language specification.
- **Transferred cleanup:** remove the legacy adjacency guard once the arity-based partition
  protects neighboring Expressions; preserve its concrete safety cases under the new behavior.
  Remove operand-position reconstruction and the second spelling classifier so positions and
  semantic kinds come from the Parser. Preserve rendering of plausible data outside Expressions
  without promoting it into executable Source; this spec selects no presentation change for those
  Cells. Pin the current visible Glyph behavior with explicit fixtures and record that preservation
  decision; do not assume the old standalone-literal exception fulfills it. These unfinished
  obligations transfer from the retired Cell-indexed tickets; they do not require the abandoned
  array representation.
- **Migration:** revise the old partial-write, competing-writer, structural-write and failed-data-
  supplier restrictions only where this contract replaces them. Preserve unrelated activation,
  partition, terminal, Sequence and platform behavior. Reconcile stale implementation descriptions
  with the final behavior as part of delivery; do not leave contradictory acceptance requirements.

## Testing Decisions

- **Primary acceptance seam:** drive the existing Source/Tick boundary from actual Source and a
  Tick. Assert Tick Plan writes, ordered Play Commands, relevant diagnostics, resulting Source and
  its next parsed interpretation. Use the existing internal fixed-Portal planning hook for cases
  that cannot yet be authored. Extend that hook at the same boundary for nested Portals, ordered
  emissions and supplied Function values, rather than exposing scheduling internals as a new API.
- **Modules exercised:** Source planning and commit integrate the Parser and Evaluator. Use focused
  existing Parser or Evaluator tests only where they isolate a real boundary rule that cannot be
  observed adequately at Source/Tick. Do not create a parallel acceptance suite for every layer.
- **Good tests:** assert externally meaningful behavior with concrete Source and independently
  specified expected outcomes. Do not assert storage shape, graph vectors, node identifiers,
  allocation count or a particular topological order when independent alternatives produce the
  same allowed result. Use overlapping outputs where Position or emission order must be observable.
- **No intermediate parsing:** retain this architectural constraint in code review. If an
  observation hook is needed to verify parse timing or exactly-once execution, keep it test-only
  and at the existing boundary; do not expose a new production tracing interface or assert a
  specific count of incremental row parses.
- **Prior art:** existing Source tests cover atomic commit, deterministic snapshots, same-Tick
  supplied operands and failed suppliers. Existing planner tests configure upward Portals, activate
  MIDI from computed Note and Bang producers, inspect explicit Tick/anchor inputs, reject cycles and
  check Cell-wise effect resolution. Parser tests cover nested arithmetic, contextual Note encoding,
  row partition and rebuild equivalence. Adapt this prior art instead of copying the HTML model.
- **Historical policy changes:** replace the failed-data-supplier test that forbids surviving data
  and the related suppressed-consumer diagnostic expectation. Split combined competing-writer/cycle
  regressions: competing writes now succeed deterministically; cycles still reject the whole Tick.
  Add the new expected behavior before removing the superseded expectations.
- **Transferred boundary regressions:** cover the exact Sources `.+01 02`, `.+0102.+0304`,
  `.+0102Z`, `***` and `.=0101 !>007FC4`, plus Comment/row bounds and unclaimed data rendering.
  Preserve complete ownership of long Expressions and row-local rebuild equivalence; do not
  reinstate the obsolete 32-record edit guard. Retain successful cleared-Bang-display rewrites.
  Cover a half-typed terminal root whose arity claims the Cells targeted by an output: a Bang in a
  typed operand must not become activation or fail silently merely because the root is inactive.
  Diagnose through the appropriate Source/claim validation boundary. Do not restore the retired
  three-policy experiment or freeze the obsolete join-error and supplier-suppression messages.
- **Property coverage:** use existing property-test infrastructure where it adds independent
  evidence: admitted write composition agrees with a simple Cell overlay; rejected complete writes
  change no Cell; incremental next-revision parsing agrees with a full parse; and fixed acyclic
  examples remain deterministic at a fixed Tick. Generate inputs within the bounded contract and
  do not use the scheduler under test to generate expected order. Do not require live-tree/next-
  parse equality: former parameter characters can become independent Source.
- **Diagnostics:** distinguish initial syntax, execution and next-revision diagnostics. A repaired
  pending operand must not contribute an intermediate execution error. Standalone result literals
  and retained parameter characters may correctly diagnose after publication. Do not require the
  entire diagnostic collection to be empty when unrelated Source intentionally remains invalid.
  A Terminal Output Function with a configured Portal must report the configuration error, even
  though it has no value to project; the rejected Portal must create no spatial dependency or write.

Required examples, derived from ADR 0034 and the accepted prototype:

| Case | Required observable result |
| --- | --- |
| Cross-boundary chain | Actual lower producers compute `01`, then `02`; a write at columns 3–4 changes `.+0101` to `.+0021`; the upper Addition answers Number `21` during T. |
| Competing writers | Computed `02` and `03` reach the same boundary in Position order; the receiver answers `31`. |
| Different producers partially overlap | Starting from `.+0101`, one producer computes `05` at column 2 and a later independent producer computes `02` at column 3. Position orders the writers; the uncovered first Cell survives, Source becomes `.+0021`, and the receiver answers `21`. Both writers must settle first. |
| Pending Note repaired | Full writes change Note input `E4 → EA → E5`; no intermediate execution error; Numeric Conversion consumes E5 and answers `4C`. Account for both Cells of each output. |
| Pending Note invalid | If `EA` remains in the Note operand, the receiver diagnoses and writes no result. |
| EA as Number | The same characters in a Number operand decode as `EA`, without inference from another literal type. |
| Whole spatial Note | A computed Note `C5` written into Addition with other operand `01` is read as Number `C5`; answer `C6`, not a sum using the MIDI value. |
| Typed nested Note | Nested Note `C5` remains typed and is rejected by Addition. |
| Dual delivery | Nested Multiply computes `0C` once; parent Addition answers `0E`; a spatial Addition consumer answers `0D`. |
| Terminal Portal configuration | A Portal on a Terminal Output Function diagnoses as invalid configuration and causes no spatial write or dependency. Ordinary activation still controls the terminal effect; validating the configuration does not activate an inactive root. |
| Rejected child write | A two-Cell `0C` output crossing the row edge writes neither Cell; the parent's answer remains `0E`. |
| Parent failure | Divide by zero produces no parent result, but its child's `0C` write survives and its spatial consumer answers `0D`. |
| Failed spatial supplier | Earlier `05` remains after a later supplier fails; receiver with other operand `01` answers `06`. Also cover failure with only original operand characters remaining. |
| Failed nested supplier | Failed child returns no typed result; its parent fails without using old parameter Source as fallback. |
| Inactive parent | Inactive terminal root and its nested child execute no computation and emit no nested Portal output or evaluation error; initial syntax diagnostics remain independent. |
| Deep nested value replacement | Computed `05` replaces Multiply and suppresses its grandchild, including both Portals; parent answers `07`; former descendant Source can become a next-Tick root. |
| Exact nested replacement | `.+02.x0304` becomes `.+02050304`; parent answers `07`; next parse claims `.+0205` and diagnoses leftover `03` and `04`. |
| Top-level value replacement | Root and child are suppressed; Source `0502.x0304` retains the former child's spelling, which first becomes a new root on the next parse. |
| Function replacement | Supplied Multiply at the original Addition anchor gives `.x0204` and answer `08`. |
| Replacement keeps nesting | Replacing the outer Addition with Multiply keeps its child and Portal; child returns `0C` once and the replacement root answers `18`. |
| Replacement signature | Replacing `.v` with `.^` gives `.^C4`; retained `C4` is Number `C4`, then fails the `00–7F` domain. Include a retained typed nested result to distinguish literal reinterpretation from typed checking. |
| Replacement arity | Unary replacement of binary Addition fails against two retained inputs during T; next parse of `.^0204` claims `.^02` and diagnoses standalone `04`. |
| Misaligned replacement | `.x` at the `+` Cell gives `..x101`; original Addition emits no result; the new anchor gets no current-Tick turn. Test the actual row extent when asserting truncation versus an operand containing space. |
| Emission order | One producer computes `05` once and emits it at columns 2 and 3; `.+0101 → .+0501 → .+0051`; receiver answers `51`. |
| Cycle with independent work | A cycle prevents every execution write and Play Command, including independent work; no partial publication. |
| Self-dependency | A write to the producer's own input diagnoses and publishes no execution effect. |

Preserve current Bang activation, no replay, terminal ordering, Sequence evaluation and rendering
regressions reachable through this change. The prototype omitted parts of that existing behavior;
its omissions are not permission to remove them.

Tracker changes must pass `node --test scripts/tests/roadmap.test.ts` and
`node scripts/roadmap.ts > /dev/null` before ticket decomposition is ready. Roadmap generation is
a local tracker gate, not merge coverage to defer to CI. At review, it failed because the release
gate omitted `language-map/09` and `language-map/10` from its dependency closure. Reconcile the
predecessor statuses, connect this release-tagged effort through `language-map/10`, and restore the
release gate's complete dependency closure. Do not suppress the error or drop release membership
to make the view render. The seven predecessor issues must have explicit, evidence-backed statuses
before `/to-tickets` treats any of their requirements as unfinished work.

For implementation, use the repository's scoped fmt, clippy and nextest gates for affected crates
and dependants, plus triggered boundary/property, feature, platform and public-interface checks.
Use the documented local property case count. Defer full merge coverage and benchmark comparison
to CI. If an implementation makes a performance claim, add a reproducible benchmark rather than
inferring speed from the prototype or storage choice. This specification itself changes no Rust.

## Out of Scope

- A Source operation that produces Function values, or new authoring syntax for arbitrary Portals.
- Destinations that change during a Tick or a general scheduler that adds/removes those dependencies.
- General-language execution semantics for newly anchored Functions during the current Tick.
- Replacement that changes activation requirements or output kind, including value-to-terminal changes.
- New variable-width spatial Sequence projection, new Sequence semantics, or changes to Bang routing.
  Preserve existing supported evaluation and activation behavior.
- A persistent typed execution store, a new Playback API, or moving expression interpretation into
  Playback or the Playback Engine.
- Selecting or optimizing a concrete storage representation, identifier width, lookup index or
  allocation strategy as a language requirement.
- Shipping the throwaway HTML or its reduced Parser/Evaluator into production.
- Treating this spec as proof of unrestricted structural execution or as a performance result.
- Performing implementation or decomposing the work into delivery tickets in this specification task.

## Further Notes

- Authority: [ADR 0034](../../docs/adr/0034-execute-against-live-typed-expressions.md), with
  [ADR 0033](../../docs/adr/0033-partition-a-row-by-parse.md) for initial and subsequent partitioning.
  ADR 0034's “Interview status and deferred questions” section records completion of the confirmed
  design interview and distinguishes the bounded contract from deferred language questions.
- Prototype primary source: local branch `prototype/integrated-live-execution-adr34`, commit
  `e0383397ba735e2677fbae8dea2d650d75be21f8`. The artifact is
  `lang/prototypes/integrated-live-execution/integrated-live-execution.prototype.html` on that branch.
  That commit records the original 24 cases. The reviewed revision on the same local branch adds
  the two explicit review cases and updated evidence. The branch has not been pushed; preserve the
  captures until the implementation context is transferred deliberately.
- Evidence: the original 24 portable-model and presentation-handler traces matched the bounded
  contract. The reviewer also reports rendering the original artifact directly from `file://` at
  1200×2600, observing the Grids, live tree, dependency table, all 24 tabs and correctly disabled
  controls. This attributed render is evidence about the prototype, not production compliance.
  The review adds explicit different-producer partial-overlap and Terminal Portal configuration
  acceptance cases; their current prototype evidence is recorded alongside the artifact.
- This spec supersedes the live-execution restrictions in the earlier
  [cell-indexed-parse spec](../cell-indexed-parse/spec.md): mandatory array layout and identifier widths,
  blanket rejection of partial/competing writes and supported original-anchor replacement, and
  preservation of old graph-error tests unchanged. Retain its completed partition work and unrelated
  regressions. Before decomposition, audit all seven issue statuses against commits and current
  code, recording shipped, abandoned and transferred scope separately. Do not execute stale
  requirements as prerequisites to this spec or treat a documentation commit as shipped code.
  The recorded audit resolves `cell-indexed-parse/01` against implementation commit `593613c`
  and marks `02–07` as `wontfix` because their historical prescriptions are superseded. Commit
  `64291cc` added planning documents only. Those six closures do not claim finished implementation:
  their retained cleanup, diagnostics and rendering obligations are carried above.
  `language-map/09` is superseded by the established arity claim; `language-map/10` remains open.
- This spec also supersedes the corresponding initial-scope restrictions and failed-spatial-supplier
  rule in [tick-execution-order](../tick-execution-order/spec.md). Preserve that effort's resolved
  history; create or refine follow-up delivery tickets rather than rewriting history as unimplemented.
- Production migration currently includes
  `a_failed_data_supplier_does_not_expose_stale_operand_cells` and
  `a_suppressed_consumer_names_the_supplier_that_did_not_settle`. These are named for discovery,
  not mandated future test names. Their old spatial-failure expectation is intentionally replaced.
- ADR 0034 makes targeted revisions to ADRs 0024, 0031 and 0032; it does not repeal their unrelated
  behavior. Dual delivery can preserve ADR 0028's value/effect distinction through Portal projection.
- Risk classification: active pre-release language design and internal Parser/Evaluator/Source
  interfaces; no public compatibility promise. No unsafe, dependency, feature or concurrency change
  is selected by this spec. Preserve existing native/WASM and feature combinations. No performance
  improvement is claimed.
- Delivery predecessor: `language-map/10` reconciles historical scheduler-ticket descriptions
  with the implemented partition and this contract. This effort belongs to `release/v1`; the
  release gate must include it in its dependency closure. Superseded Cell-indexed mechanisms are
  not implementation prerequisites.
- Next step: `/to-tickets` after the predecessor audit is recorded and the local tracker gates pass.

The review reconciliation passes both tracker commands: roadmap tests (10 cases) and roadmap
generation. Future `.scratch/` changes must keep those gates green; the former release-closure
failure is fixed, not an accepted exception for `/to-tickets`.

2026-09-08: Implemented by issues 02–10; see [production delivery evidence](evidence.md). Historical prototype and planning observations above remain attributed to their original stage.
