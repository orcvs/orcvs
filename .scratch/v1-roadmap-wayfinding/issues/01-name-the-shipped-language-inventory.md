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

The First Release Candidate ships the complete accepted numeric, Sequence, Tick, spatial, and MIDI
slice below. Release membership follows this inventory, not mere appearance in `CONTEXT.md` or ADR
0019. “Satisfied” means the current implementation substantially provides the release behavior;
“partial” and “missing” remain implementation work even where syntax or a legacy approximation
already exists.

### Values and Source forms

| Inventory member | FRC contract | Current state |
| --- | --- | --- |
| Number | Contextual two-Cell uppercase hexadecimal `00`–`FF`; wrapping general arithmetic | Substantially satisfied |
| Note | Contextual canonical pitch spelling `C/`–`G9`, carrying MIDI `00`–`7F` | Partial: the current tables omit MIDI 0–20 |
| Bang `**` | One-Tick Atom with deterministic activation and expiry | Syntax partial; Tick behavior missing |
| Activation Characters `^^`, `vv`, `<<`, `>>` | Direction-preserving Source units with one-Cell-per-Tick movement and collision-to-Bang behavior | East syntax only; behavior missing |
| Sequence | Flat ordered, non-nesting Atom value; compatible Atomic Functions extend pervasively | Missing |
| Comment | `##` through row end; lone `#` is incomplete or invalid | Missing pending Language Map |

Function and Character implementation variants and `Empty` sentinels are not automatically shipped
language values. Bangs may inhabit Sequences: structural operations preserve, select, or replace
them, while only explicitly compatible Atomic Functions may evaluate them. Bangs remain invalid
Number or Note Range bounds.

### Numeric family

The FRC ships the complete accepted family: Addition `.+`, ordered Subtraction `.-`, Absolute
Difference `.|`, Multiplication `.x`, Division `./`, Modulo `.%`, Minimum `.<`, Maximum `.>`,
Equality `.=` and explicit Note-to-Number `.v` / Number-to-Note `.^` conversion.

`.+`, `.-`, `.x`, and `./` are substantially implemented. `.v` and `.^` have core evaluation but
remain partial until the full Note domain and Sequence extension are satisfied. `.|`, `.%`, `.<`,
`.>`, and `.=` are missing.

### Sequence family

The FRC ships Number Range `:-`, Note Range `:#`, Reverse `:<`, Concatenate `:&`, Select `:?`, and
Replace `:=` with the accepted flat-Sequence behavior. All are currently missing. Replace is a pure
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
chains over complete aligned Language Units, snapshot reads, atomic writes, deterministic effect
ordering, and diagnostics. Jump does not transport a Sequence or partial Language Unit. This
behavior is currently missing apart from limited lexical spellings.

### MIDI terminal-output family

The FRC ships all five accepted MIDI forms:

- Raw Play `!>` with channel `00`–`0F`, velocity `00`–`7F`, and typed Note.
- Timed Play `!~` with explicit length, zero rules, and Note Off at Tick `T + length`.
- Monophonic Play `!%` with one owned voice per output adapter and MIDI channel.
- Control Change `!c` with direct controller/value data bytes.
- Pitch Bend `!b` with direct LSB then MSB data bytes.

Raw Play is partial: parsing, a command shape, and native delivery exist, but channel validation,
the complete Note domain, activation, and Tick Plan integration remain incomplete. Timed Play,
Monophonic Play, Control Change, and Pitch Bend are missing.

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
