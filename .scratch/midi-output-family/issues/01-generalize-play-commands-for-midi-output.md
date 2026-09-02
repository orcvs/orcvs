# 01 — Generalize Play Commands for MIDI output

**What to build:** Replace the single raw-note command shape with explicit ordered MIDI command
variants capable of carrying ADR 0016's terminal outputs without leaking MIDI byte assembly into
Source interpretation.

**Blocked by:** orcvs-language-migration/02; orcvs-language-migration/04; lang-foundations/06.

**Status:** ready-for-agent

**Tags:** release/v1

- [x] Raw Play uses `!> channel velocity note` and emits only when its root is active.
- [x] Channel, velocity, Note, and later data-byte domains diagnose before command creation.
- [x] Terminal Functions remain invalid as nested value-producing Functions.
- [x] Tick Plan retains command producer order.
- [x] Tick planning, not MIDI interpretation, owns whether a root is active; integration coverage
      proves inactive terminal roots emit nothing and active roots retain Tick Plan order.
- [x] Playback/output adapter seam accepts explicit commands without parsing Source intent.
- [x] Existing raw Note On delivery and velocity-zero behavior remain covered.

## Comments

Implemented. `PlayCommand` is now an enum with the single `Raw { channel, velocity, note }`
variant; MIDI byte assembly stays in `MidiOutputAdapter::submit`. Domain checks moved to the
shared `functions::midi_channel` and `functions::midi_data_byte(role, value)` helpers, whose
diagnostics read `MIDI channel {value} is outside the range 00–0F` and
`MIDI {role} {value} is outside the range 00–7F`, so `!c` and `!b` reuse them by passing a
different role. `Function::is_terminal()` is generated from the canonical `define_functions!`
table and drives both the Interpreter's nesting guard
(`InterpretationError::NestedTerminalFunction`) and the new activation gate in
`Source::plan_tick`. `LanguageMap::is_root_active` answers the ADR 0006 geometry from the Bang
outward, and an inactive terminal root is skipped before evaluation, so it emits neither a
command nor a diagnostic.

Known limitation, deliberately left for `spatial-tick-planning/01`: a Bang horizontally
adjacent to a root cannot activate it today. `row_extents` splits Expression runs only on
spaces and `##`, so `**!>007FC4` is one run that parses as a Bang followed by unexpected
trailing content and forms no root at all; the same applies to a Bang immediately east of a
root's Footprint, which the parser reads as trailing content of that Expression. The west and
east anchors are implemented and unit-tested against the Language Map directly
(`a_bang_activates_the_root_anchor_at_each_of_its_four_cardinal_positions`), while the
end-to-end Tick tests use vertical placement, which the current Expression partition supports.

Diagnostics are gated along with emission, deliberately. An inactive terminal root is skipped
before `Interpreter::execute`, so it produces no domain or arity diagnostic either: before this
change a lone `!>0080C4` reported `MIDI velocity 80 is outside the range 00–7F` on every Tick,
and now it is silently unvalidated until a Bang reaches it. That follows from ADR 0006 — an
ordinary root Expression is inert until Bang activation, and an evaluation-time diagnostic is an
outcome of evaluation, so an Expression that never evaluates has no outcome to report. The
Language Map's lexical and syntax diagnostics are unaffected and still fire regardless of
activation, so a malformed Play is still flagged as one. The checklist gates emission; this
extends the same gate to the diagnostics emission would have produced.

Known cost, deliberately left unoptimized: `LanguageMap::is_root_active` re-scans every Language
Unit for every terminal root, so activation is O(roots x units) per Tick. A precomputed
Bang-anchor set would have to be built after `LanguageMap::build`'s reclassification pass, which
rewrites `unit.kind` — caching it now would add ordering fragility for a gain no benchmark shows
matters, and `spatial-tick-planning/01` rebuilds this path anyway.

Also out of scope here and left to `spatial-tick-planning`: Bang one-Tick expiry (a
Source-resident Bang persists across Ticks, so an activated terminal root repeats every Tick),
gating value-producing roots, Self-Banging Functions, Halt locks, and the row-major producer
pass.
