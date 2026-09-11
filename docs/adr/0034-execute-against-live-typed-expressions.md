# Execute against live typed expressions

Status: accepted. Production acceptance and verification are recorded in the [delivery evidence](../../.scratch/live-typed-execution/evidence.md).

The Parser's output is the typed expression structure used by rendering and Tick execution. Source locations belong to that output; a separately interpreted Language Map is unnecessary. Parse operates on Source only. During a Tick, execution order threads earlier Function outputs into later Functions' parameters at their grid positions; Source parsing is not an intermediate execution stage.

## Decisions established in discussion

- At Tick T: parse Source, execute Functions in dependency order while threading execution outputs through the call tree, then render the resulting state as Source for Tick T + 1. Source parsing does not run between Function executions.
- Order Function execution by output dependencies, and have scheduled entries refer to the live expression structure.
- If Function B outputs over or at Function A, B must execute before A. An output reaching an already-executed Function is a bug in the ordering, not a supported case requiring re-execution or delayed propagation.
- Nested Function results pass typed values to their consumers. Spatial outputs always act as literal Source encodings, including when they cover a whole parameter. The receiving operand interprets the resulting characters according to its declared literal type. A Note output spelled `C5` reaching an Addition operand therefore becomes Number `C5` (hexadecimal, decimal 197); the same Note returned by a nested Function remains a Note and is rejected by Addition. This is contextual interpretation of an encoding, not conversion to the Note's underlying MIDI number. Operand decoding does not rerun Source partitioning between executions.
- A Function can have both a typed result for nesting and a spatial output through a Portal. It computes once, passes its typed result to its parent, and emits its spatial output during that same execution. Both consumers depend on that computation. This decision does not introduce a Function spelling.
- A self-dependency or dependency cycle prevents publication of all execution effects for that Tick, including those from independent computations. Report the cycle; do not publish partial execution or silently delay dependencies to another Tick.
- A nested Function's Portal does not independently activate that Function. An inactive parent or replacement of the nested computation suppresses the child's execution and therefore both its nested result and spatial output.
- Failure to validate a spatial write's complete destination rejects that write and reports a diagnostic, but preserves the Function's successful typed result for its parent. For example, Multiply can return Number `0C` even when its two-Cell spatial output would extend beyond the Grid. No part of the rejected write is applied.
- A producer's evaluation error reports a diagnostic and supplies no spatial write. Its spatial consumers proceed after it settles, using the operand encodings that remain, including successful earlier writes. The failed producer does not invalidate or erase those encodings. A nested consumer instead fails when its child supplies no typed result. This distinction follows the two delivery paths: spatial consumers read surviving characters, while nested consumers require a returned value.
- A parent's evaluation error does not undo its child's successful spatial output. For example, in `Divide(Multiply(3, 4), 0)`, a Multiply with a Portal returns Number `0C` and spatially writes `0C`. Divide then diagnoses division by zero and produces no result; Multiply's write remains in the Tick's output. Dependency cycles instead reject all execution effects, as specified above.
- Spatial writes compose Cell-wise in live operand encodings, just as they do in Source. Later writes replace covered Cells and preserve earlier writes outside that coverage. Dependency order determines execution, Position breaks ties, and a Producer's emission order orders its own writes. All writers must settle before the receiving Function executes.
- Spatially updated operands carry pending encodings until the receiving Function executes. Writes do not decode those encodings or produce intermediate decode failures. At execution, the receiving operand's declared literal type determines decoding, followed by the Function's type and domain checks. In a Note operand, `E4` changed to `EA` and then `E5` before execution yields Note `E5` without an evaluation error for `EA`. If `EA` remains when consumed as a Note, execution diagnoses and produces no result. Pending characters are not assigned another type merely because that type could decode them: `EA` is valid in a Number operand, but invalid in a Note operand. Whole-destination validation still precedes each write. This rule concerns execution-time operand decoding; it does not rerun or replace initial Source partitioning.
- Replacing a Function with another Function retains the original parameters and connections. It does not recruit additional Source cells, discard parameters, or reshape the expression to fit the replacement signature.
- Replacement is positioned by the Grid, without an alignment requirement against parsed items. Output `.x` at the `+` cell of `.+0101` gives `..x101`; the original Addition is discarded. The malformed resulting arrangement is allowed, rather than refusing the output to preserve the parse. Retaining parameters applies to a Function replacement at its original anchor; it does not establish how newly aligned Function spellings in an overlapping output execute.
- Parameter updates respect the cells covered by an output, even across original parameter boundaries. Given `.+0101`, an earlier output `02` at cells 3–4 changes the displayed arrangement to `.+0021`. The later Addition executes with Number `00` and Number `21`, answering Number `21` (hexadecimal, decimal 33) in the same Tick. Neither refusing the overlap nor invalidating the parameters until the next revision is the rule. Production keeps the covered operand characters pending and decodes them at consumption without invoking Source parsing during execution.
- The replacement Function is checked against those retained parameters when it executes. Replacing Addition in `.+0204` with Multiply gives `.x0204` and an answer of `08`, provided the replacement precedes that expression's turn. An incompatible arity or type gives an error.
- At an original-anchor Function replacement, the replacement's signature determines literal interpretation at execution, including for retained literal characters that no spatial write changed. Retaining parameters means retaining their positions and connections, not freezing their initial literal types. For example, replacing `.v` with `.^` in `.vC4` gives `.^C4`: `C4` is now Number `C4`, outside `.^`'s accepted `00`–`7F` range, so execution diagnoses. A nested result remains typed and is checked as such; it is not reinterpreted as literal characters.
- A value replacing a nested Function replaces that computation as an input: `Add(2, Multiply(3, 4))` receiving Number 5 at Multiply becomes `Add(2, 5)`. Its former parameters are unused. A value replacing a top-level Function suppresses its turn and produces no further output from that former Function. Former parameter characters remain unless a write overwrites them.
- Adding a Source operation that produces Function values is deferred.
- Source remains the editable character representation. Typed execution state is local to a Tick; the character revision is parsed for subsequent interpretation.
- Storage representation and optimization are deferred. Neither individually allocated linked nodes nor a cell-indexed classification array is a settled requirement.
- The bounded production implementation uses fixed destinations and includes nesting, nested value replacement, and Function replacement at an original anchor, including replacement that changes literal signatures. It excludes replacements that change activation requirements or output kind, such as replacing a value Function with a MIDI terminal Function. Changed destinations and execution of newly anchored Functions are also deferred beyond this experiment; their general language behavior is not settled by these scope limits.

These decisions revise the spelling-only Map separation in [ADR 0024](0024-the-language-map-records-spellings-not-atom-types.md) and the working-character binding seam in [ADR 0031](0031-evaluate-turns-against-working-source.md). The production migration retires the independent spelling classifier and reconstructed binding layout. They lift [ADR 0032](0032-schedule-tick-execution-by-dependency.md)'s initial restrictions against partial input projections and competing writers, while retaining its distinction between contextual spatial decoding and typed nested results. Original-anchor Function replacement also revises its preservation of parsed operand types: the replacement signature interprets retained literals. These are targeted revisions, not a wholesale removal of its structural restrictions. [ADR 0033](0033-partition-a-row-by-parse.md) continues to determine the initial parse from Source.

The failed-spatial-producer rule revises the [existing scheduler specification](../../.scratch/tick-execution-order/issues/02-schedule-current-tick-dependencies.md), which forbids exposing surviving operand data after supplier failure. Production now follows the revised rule; the former failed-data-supplier and suppressed-consumer tests have been migrated to assert surviving Source and successful receiver results.

The sentence that accompanied the deferral of a Source operation producing Function values, permitting the execution design to be tested with supplied typed Function values instead, is struck. It arrived in commit `7c7554d` together with the `cfg(test)` field it licensed, so it recorded a decision at the moment the code was written rather than deciding anything before it, and what it licensed was a shipped scheduler and executor that compiled differently under test: a substitution replacing an interpreted answer, and a reservation width a computation could take from outside the Function table. The deferral itself is unchanged — nothing in Source produces a Function value — and so is every rule above that a Function value reaches, including replacement at an original anchor and the refusal of a replacement that changes activation requirements or output kind. A test that needs an input production cannot produce now constructs that input below the planning entry point and delivers it through the shipped path, which leaves the module a test exercises the module a release contains.

The spatial encoding rule replaces this proposal's earlier rule that whole spatial replacements preserve the producer's type. Whole and partial spatial writes now use the receiving operand's literal interpretation. The reason is Source-equivalent behavior: identical final operand characters have the same meaning, whether entered directly or supplied spatially. Historical prototype assessments are evidence about their recorded candidate, not the production contract.

## Amendment: the Render Frame carries Tokens, not a Glyph classification

This ADR states that the Parser's typed output is the structure rendering uses and that the
production migration retires the independent spelling classifier. One classifier survived the
migration: `orcvs::glyph::Glyph`, the per-Cell enum the Render Frame carries and the console paints
from. It is retired too, and the Render Frame carries `Option<Token>` in its place.

`Glyph` is not a paint vocabulary that resembles the token vocabulary. It is that vocabulary with
the type-carrying variants removed:

| Carried by | Variants |
| --- | --- |
| Both | `Bang`, `Char`, `Comment`, `Function`, `Note`, `Number` |
| `Glyph` alone | `Marker`, `Highlight`, `Space` |
| `Token` alone | `Activation`, `Atom`, `Sequence` |

`Space` is the absence of a Token, which `Option` already spells. `Marker` and `Highlight` are the
Orca inheritance: in a terminal the Grid's background rulings *are* characters, so a `+` every
Marker Spacing and a `.` at the Highlight Spacing were Cell content. Orcvs paints both as geometry
instead — `marker_spacing` drives the sector seam strengths and `highlight_dot_spacing` the Cursor
bloom radius — and no Cell has been able to carry either classification since. `LanguageMap` assigns
a Glyph only to a Cell holding a parsed token or a non-space byte, so every other Cell answers
`Glyph::Space`, and `GlyphString::marker()` and `::highlight()` are reached by nothing but their own
unit test.

That leaves `Activation`, `Atom` and `Sequence`, which `From<Token> for Glyph` folds into `Char`
with a comment recording the loss. The fold is the whole of what the classifier does that `Option`
does not, and what it discards is exactly the type information this ADR established: a generic Atom
operand, a whole-Sequence operand and a literal character become one colour.

The decision:

- `RenderCell` carries `Option<Token>`. `Glyph`, `GlyphString`'s blank spelling table,
  `From<Token> for Glyph`, and the `LanguageMap` row's `Vec<Option<Glyph>>` are deleted. This ADR
  already deferred that array's existence — "neither individually allocated linked nodes nor a
  cell-indexed classification array is a settled requirement" — so removing it settles the deferral
  rather than revising a decision.
- `Marker` and `Highlight` leave the vocabulary. `CONTEXT.md`'s **Glyph** entry is retired with the
  type it named, and the Grid's background rulings are described where they are drawn.
- Facts that belong to an Expression stay on the Expression. A diagnostic and an Expression's
  executability are per-Expression and already live on `ExpressionEntry` as `diagnostic`, `span` and
  `root`; the Render Frame carries those spans once rather than copying two booleans onto every Cell
  the Expression covers. Copying them per Cell would repeat at a finer granularity the duplication
  this amendment removes.

What this does not decide, because it is language design rather than a consequence of retiring a
duplicate, is deferred to `.scratch/typed-source-paint/`:

- What an unfilled operand Cell shows. `GlyphString` spells `h` for an empty Number slot and `n` for
  a Note, which is the same terminal inheritance as `Marker`: standing a character in because a
  terminal has only characters. A typed console can paint the Cell in its declared type instead and
  spell nothing. Neither is adopted here.
- How a diagnostic and an inexecutable Expression are painted. That they *can* be is the point of
  this amendment; which paint they take belongs with the palette record.

## Interview status and deferred questions

The bounded integration design was confirmed in discussion and subsequently implemented through the production Source/Tick boundary. [Delivery evidence](../../.scratch/live-typed-execution/evidence.md) maps all 26 acceptance cases to regressions and records local verification. Prototype approval alone was not acceptance of production behavior; the captured HTML remains outside the implementation.

The following general-language questions are deferred beyond that experiment, not adopted policies:

- What happens when an overlapping Function-valued output establishes a Function spelling at a different anchor? Function-valued output is deferred; this is not a prerequisite to current parameter propagation.
- How should Function replacement change activation requirements or output kind?
- How should changed destinations introduce or remove dependencies during a Tick?

General replacement that changes activation or output behavior remains unresolved. Production tests verify the established error rules and retention of untouched Source characters for the next revision's arity-based parse. The parse / ordered execution / render sequence is settled. The bounded contract does not define unrestricted structural execution.
