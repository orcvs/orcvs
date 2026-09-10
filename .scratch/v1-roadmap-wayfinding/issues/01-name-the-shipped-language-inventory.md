# Name the shipped language inventory

Type: grilling

Blocked by: None — can start immediately.

Status: resolved

## Question

Exactly which accepted Orcvs values, Function families, spatial behaviors, Tick behaviors, and
terminal outputs constitute the First Release Candidate, which are already satisfied by current
implementation, and which remain explicit deferrals? Produce one authoritative inventory that can
cross-check `CONTEXT.md`, ADR 0019, implementation tickets, and release evidence without treating
every term in the evolving glossary as automatically shipped.

## Answer

**State column refreshed 2026-09-09 against `f131e5e`.** The original column was written against
`bbe882e` (2026-09-01). Nine members moved after it, so the release-candidate audit restated each
one. The FRC contract column is unchanged.

The First Release Candidate ships the complete accepted numeric, Sequence, Tick, spatial, and MIDI
slice below. Release membership follows this inventory, not mere appearance in `CONTEXT.md` or ADR
0019. “Satisfied” means the current implementation substantially provides the release behavior;
“partial” and “missing” remain implementation work even where syntax or a legacy approximation
already exists.

### Values and Source forms

| Inventory member | FRC contract | Current state |
| --- | --- | --- |
| Number | Contextual two-Cell uppercase hexadecimal `00`–`FF`; wrapping general arithmetic | Substantially satisfied |
| Note | Contextual canonical pitch spelling `C/`–`G9`, carrying MIDI `00`–`7F` | Satisfied: conversion is algorithmic and total over `00`–`7F` (`lang/src/lib.rs`) |
| Bang `**` | One-Tick Atom with deterministic activation and expiry | Substantially satisfied: activation, producer-before-consumer ordering, once-only execution, stale-display clearing, and parser-owned validity ship through the ADR 0032/0034 scheduler. East and west anchor coverage and direct delivery remain (`spatial-tick-planning/02`) |
| Self-Banging Functions `^^`, `vv`, `<<`, `>>` | Root-only Source Functions with intrinsic Bang activation, one-Cell-per-Tick Portal writes, and collision-to-Bang behavior | All four spellings parse (`lang/src/atom.rs`, `lang/src/parser.rs`); movement and emission missing |
| Sequence | Flat ordered, non-nesting Atom value; compatible Atomic Functions extend pervasively | Value and pervasive broadcasting ship (`lang/src/sequence.rs`, `lang/src/stack.rs`); unreachable from Source until `:-` or `:#` can spell one, and a Sequence result cannot reach Source (`sequence-values/06`) |
| Comment | `\|\|` through row end; lone `\|` is incomplete or invalid | Shipped (`lang/src/parser.rs`); `sequence-values/07` respelled the introducer and moved the rule out of the byte pre-pass into the parse |

Function and Character implementation variants and `Empty` sentinels are not automatically shipped
language values. Bangs may inhabit Sequences: structural operations preserve, select, or replace
them, while only explicitly compatible Atomic Functions may evaluate them. Bangs remain invalid
Number or Note Range bounds.

### Numeric family

The FRC ships the complete accepted family: Addition `.+`, ordered Subtraction `.-`, Absolute
Difference `.|`, Multiplication `.x`, Division `./`, Modulo `.%`, Minimum `.<`, Maximum `.>`,
Equality `.=` and explicit Note-to-Number `.v` / Number-to-Note `.^` conversion.

The complete family is implemented. `.+`, `.-`, `.x`, and `./` were already substantially
implemented; `.|`, `.%`, `.<`, `.>`, and `.=` have since shipped, and both conditions that held
`.v` and `.^` partial — the full Note domain and pervasive Sequence extension — are now met. All
eleven appear in `define_functions!` (`lang/src/atom.rs`), and the arithmetic laws run exhaustively
over every byte pair (`lang/src/interpreter.rs`, `lang/src/functions/math.rs`).

### Sequence family

The FRC ships Number Range `:-`, Note Range `:#`, Reverse `:<`, Concatenate `:&`, Select `:?`, and
Replace `:=` with the accepted flat-Sequence behavior. All six are currently missing. Two
prerequisites are also missing and are tracked in `sequence-values/06`: the executor refuses every
Sequence result, and a signature cannot name a generic Atom or Sequence operand. Replace is a pure
value operation; the general Source Write that may follow it remains deferred. Complete-fit atomic
Sequence result delivery through Portals is release infrastructure, not a shipped Portal value.

### Tick and feedback family

The FRC ships Clock `~.`, Delay `~*`, visible Increment `~+`, deterministic Random `~?`, Euclidean
rhythm `~%`, and visible Interpolation `~>`, including the accepted Tick-zero, feedback, and
determinism boundaries. All are currently missing.

### Spatial and Tick-planning behavior

The FRC ships Directional Bang Functions `*^`, `*v`, `*<`, `*>`; Halt `*!`; and directional Jump
Functions `&^`, `&v`, `&<`, `&>`. It includes Bang routing and expiry, Activation movement and
collision, Source-order root turns, later-root same-Tick activation, Halt locking, directional Jump
chains over complete aligned Language Units, atomic writes, deterministic effect ordering, and
diagnostics. Jump does not transport a Sequence or partial Language Unit. Bang routing, expiry, and
effect ordering now ship through the dependency scheduler; ADR 0032 replaced Source-order root
turns with dependency order, where Position only breaks ties. Directional Bang Functions, Halt, and
Jump remain missing — no `*^`, `*v`, `*<`, `*>`, `*!`, `&^`, `&v`, `&<`, or `&>` appears in
`lang/src` or `orcvs/src`.

### MIDI terminal-output family

The FRC ships all five accepted MIDI forms:

- Raw Play `!>` with channel `00`–`0F`, velocity `00`–`7F`, and typed Note.
- Timed Play `!~` with explicit length, zero rules, and Note Off at Tick `T + length`.
- Monophonic Play `!%` with one Playback-Engine-owned voice per MIDI channel.
- Control Change `!c` with direct controller/value data bytes.
- Pitch Bend `!b` with direct LSB then MSB data bytes.

All five ship. Raw Play's four recorded gaps are closed: channel and velocity carry typed MIDI
domains, the Note domain is complete, terminal activation gates execution, and Tick Plan
integration is in place. Timed Play, Monophonic Play, Control Change, and Pitch Bend have since
shipped (`lang/src/atom.rs`; `midi-output-family/02`–`04`). The residual is Bang expiry and the
narrow safety action — CC 123 only, with CC 121 and the centred bend open in
`midi-output-family/06`.

### Explicit deferrals and omissions

- General Source Read `@<` and Source Write `@>` addressing, Generator composition, and visible
  Konkat-style reads. Directional Jump is the only shipped Address subset.
- UDP `!u`, OSC `!o`, and their text/message values and transports.
- Application Command `!$` and its command value encoding.
- Cross-version Source, persistence-format, and Rust-interface compatibility while Orcvs remains
  pre-release.
- Improvement-only maintenance that is not required for correctness, safe implementation, or
  measured release evidence.
- Hidden-variable behavior is omitted rather than deferred; Identity Test is retired.

The Language Map, compiler-checked Function definitions, typed operand extraction, canonical Source
generation, Tick planning, and target evidence are prerequisites for proving this inventory, but
they are infrastructure or proof work rather than additional shipped language members.
