# 01 — Trim glossary entries to definitions

**Status:** needs-triage

**What to do:** Many `CONTEXT.md` entries still carry operational policy that the requirements now
own: Pending, Defaulted Operand, Return, Comment (LANG); Tick, Turn, Dependency, Reservation,
Contact, Effect, Function Replacement, Source Bundle, Halt, Self-Banging and Bang Functions, Tick
Portals, Live Editing, and the Clock, Every, Euclidean, Increment and Slew entries (EVAL); Playback
Engine, Monophonic Play, Timed Play, Output Command, Tick Grid, Overrun, Rewind, Run Clock and
Omitted Diagnostics (PLAY); Theme, Paint, Cursor and Sector Seam (CONS). Trim each to its
definition, distinctions and `_Avoid_` line, leaving behaviour to the owning requirement.

**Why not in the snapshot change:** Rust comments cite several entries by name, and trimming them
needs those citations kept resolvable or repointed:

- `console/src/function_reference.rs:56`, `:98`, `:982` cite the Halt and Terminal Output Function
  entries.
- `orcvs/src/source/tick/execution.rs:630` cites Function Replacement's four-fact sentence.
- `orcvs/src/source/encoding.rs:3` cites Spatial Output.

The snapshot change trimmed only the Built-in Synth entry, which no code cites, and corrected the
Function, Clear, Pan, Operand Literal and Playback Engine (refused submission) entries.

**Done when:** each listed entry holds no rule a requirement states, every code citation still
resolves, and `docs/requirements/` is unchanged in meaning.
