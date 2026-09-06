# Schedule Tick execution by dependency

Status: accepted. Supersedes ADR 0031 and the row-major execution, snapshot-only operand reads, and stored-Bang activation clauses of ADRs 0003, 0004, 0006, 0009, 0012, 0014, and 0020.

When Functions supply a Note and a neighboring Bang to MIDI during Tick T, that Note must play during T, including when a producer appears below its consumer. Parse the program, establish data and activation dependencies, order the operations, execute each scheduled root at most once, and publish the Tick outcome. Grid position breaks ties between independent operations; it does not override a dependency. A fixed upward Portal is an ordinary dependency, not delayed feedback.

Parsing owns Function identities, operand types and positions, and contextual Bang validity. Preserve that structure, including missing or invalid operand slots that current-Tick writes can supply. Nested Functions remain inside their containing Expression and use the existing Evaluator. Scheduling and current operand resolution stay behind the Source Tick interface; Playback receives the ordered musical commands for T.

A Function's Bang result activates neighboring roots during its production Tick. `**` is its transient visual representation; manual `**` is a no-op, and the displayed result does not activate again on T+1. Bang is a value, not a standalone Function. A `**` rejected in a typed operand is still invalid syntax and neither activates nor receives display cleanup. The initial implementation can discard valid prior Bang display before evaluation without recording pulse-age metadata.

Spatial writes preserve Cell encoding semantics: the receiving operand decodes the projected encoding according to its parsed type. Nested evaluation continues to pass typed values. A potential supplier must settle before its consumer, even when its result is absent; absence of Bang means no activation, not an unfinished dependency. An empty result performs no write and preserves existing ordinary operand data.

## Initial scope and consequences

Implement fixed destinations and whole-slot scalar projections first. Diagnose competing writers, same-Tick dependency cycles, partial input projections, and unsupported structural writes before publishing affected execution. The accepted prototype rejects the whole Tick for those graph errors; it does not invent writer priority, silently delay a cycle, or repeatedly evaluate roots. Generated Function code cannot alter the graph already being executed. Arbitrary computed destinations, variable-width Sequence projection, explicit delayed feedback, and same-Tick structural mutation require subsequent decisions.

Preserve the Source/Playback separation and atomic Tick publication from ADR 0001. The first implementation retains the current Function-family activation policy—value roots calculate each Tick and terminal roots require a Bang—while making their ordering and Bang lifetime correct. This decision does not implement the broader directional movement, Jump, or Halt designs; those tickets must adopt the dependency model before adding their effects.

The bounded prototype demonstrated same-Tick C4 with producers above and below MIDI, the exact `Y-1` Bang Portal, a calculation chain, absence, conflicts, cycles, invalid operands, manual no-op, and no replay. See [the exploration](../../.scratch/language-map/operation-ordering-exploration.md) and [implementation tickets](../../.scratch/tick-execution-order/spec.md).
