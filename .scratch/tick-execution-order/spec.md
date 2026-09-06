# Execute a musical Tick in dependency order

Status: resolved

## Goal

Implement [ADR 0032](../../docs/adr/0032-schedule-tick-execution-by-dependency.md): Note data and Bang activation produced during T reach their consumer during T regardless of producer position. `**` is output display, manual `**` is inert, and old display never replays.

## Delivery order

1. [01 — Preserve parser-owned input layouts](issues/01-preserve-parser-owned-input-layouts.md).
2. [02 — Schedule and execute current-Tick dependencies](issues/02-schedule-current-tick-dependencies.md).
3. [03 — Integrate playback and verify the execution contract](issues/03-integrate-and-verify-tick-execution.md).

These replace the provisional row-major implementation. They also carry the implementation needed to close `language-map/06` and `spatial-tick-planning/02`; those existing issues retain their originating context and acceptance history.

## Boundaries

Use fixed Portal destinations and whole-slot scalar writes. Keep Source's Tick interface and the existing Expression Evaluator. Preserve encoded Cell transport and parser authority. Diagnose competing writers, cycles, partial operand writes, and structural projections unsupported by the stable graph. Do not silently choose a previous-Tick input, introduce a standalone Bang Function, or add a separate pass that reclassifies raw spellings.

No new authoring syntax for arbitrary Portals is selected. Exercise fixed upward routing through the internal destination mechanism. Dynamic addresses, variable-width Sequence projection, feedback delay syntax, moving Functions, Jump, Halt, and broader Function-family activation rules are follow-up scope.

## Current state

The design and nine-scenario HTML exploration are accepted. Production integration is paused while these tickets are recorded. The worktree contains the earlier row-major fallback plus partial parser and generated-pulse fixture edits; no ticket is resolved by those drafts. See each ticket's Comments for evidence and remaining work.

Reference: [exploration and prototype](../language-map/operation-ordering-exploration.md).
