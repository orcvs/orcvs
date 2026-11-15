# 01 — Trim glossary entries to definitions

**Status:** needs-triage

**What to do:** Trim every `CONTEXT.md` entry to its definition, its distinctions and its `_Avoid_`
line. Move what it states beyond that to where it belongs:

- A rule about what the product does goes to the owning requirement in `docs/requirements/`, or is
  dropped where that requirement already states it.
- A per-Function spelling, operand list or worked example (such as `&t0103C4D4E4` reads `D4`, or
  `#pC4MD05` answers A5) goes to `docs/functions.md`, or is dropped where it is already there.
- A rule-ID pointer such as "(PLAY-180)" goes with the clause it supports.

The scope is every entry. These carry the most behaviour and are a seed list, not the boundary:

- Language: Interval Set, Defaulted Operand, Return, Comment, Clear, Arithmetic Function, Pitch
  Function, Pick, Stack, Random Function, Copy Function, Read Function, Track Function.
- Evaluation: Tick, Turn, Dependency, Reservation, Contact, Effect, Output Portal, Input Portal,
  Function Replacement, Source Bundle, Bang, Missed Bang (it quotes a diagnostic string), Halt,
  Self-Banging Function, Bang Function, Tick Portals, Live Editing, Clock, Every, Euclidean,
  Increment and Slew.
- Playback and outputs: Playback Engine, Tick Grid, Overrun, Rewind, Run Clock, Omitted
  Diagnostics, Output Command, Terminal Output Function, Raw Play, Timed Play, Monophonic Play,
  Raw Chord Play, Timed Chord Play, Control Change, Pitch Bend, Application Command, UDP, OSC,
  Payload.
- Console: Theme, Paint, Cursor, Sector Seam, Info.
- Site: Sound Module.

**Code citations:** every Rust comment that cites `CONTEXT.md` now cites it for a term, not a rule,
so trimming an entry leaves each one resolving. Recheck with
`grep -rn "CONTEXT.md" --include=*.rs lang orcvs console`.

- Repointed to the rule ID that owns the rule (done):
  - `orcvs/src/source/language_map.rs:2009`, `:2231` → LANG-060; `:2701` → LANG-080.
  - `lang/src/parser.rs:2015` → LANG-080.
  - `orcvs/src/grid.rs:986` → LANG-020, LANG-030.
  - `orcvs/src/source/portal.rs:15` → EVAL-030; `:63` → EVAL-053, EVAL-054.
  - `orcvs/src/source/tick/halt.rs:91` → EVAL-053.
  - `orcvs/src/source/tick/execution.rs:630` and `lang/src/atom.rs:1440` → EVAL-040.
  - `console/src/function_reference.rs:56` → EVAL-053; `:98`, `:982` → LANG-180, PLAY-340.
- Kept, because they cite a term's definition:
  - `orcvs/src/source/language_map.rs:2032`, `orcvs/src/grid.rs:1028`, `lang/src/parser.rs:1542`
    (Cell).
  - `lang/src/atom.rs:502` (Effect).
  - `orcvs/src/source/encoding.rs:3` (Spatial Output).

**Needs a decision first:** Activation is "what lets a root take its Turn", as EVAL-050 says a
Halt or Terminal Output root takes a Turn only when a Bang activates it. The Halt entry says a Halt
above its own `**` "takes its Turn inert", as EVAL-053 implies, and Terminal Output says "Activation
gates evaluation itself". Settle in `docs/requirements/evaluation.md` whether an unactivated root
takes a Turn before trimming those entries.

**Done when:** no `CONTEXT.md` entry states a rule that a requirement or `docs/functions.md` states,
checked against every entry rather than only the seed list; every code citation of `CONTEXT.md`
still names a term the glossary defines; and `docs/requirements/` is unchanged in meaning.
