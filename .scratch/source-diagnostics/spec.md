# Present Source and Tick diagnostics in the console

Status: ready-for-agent

## Problem

The console shows no Source or Tick diagnostic messages (ADR 0069). Language Map diagnostics reach the Render Frame (`orcvs/src/render_frame.rs`) and colour an invalid operand slot, but their messages are never shown. Tick diagnostics are collected in `TickPlan.diagnostics` (`orcvs/src/source/model.rs`) and the Playback Engine drops them (`execute_tick` in `orcvs/src/playback.rs`). An invalid Function writes nothing and leaves its last answer standing (ADR 0069), so a value Function between Track and a Play that goes invalid replays its last Note, which a performer cannot tell apart from a working patch.

## Decisions

Settled 2026-10-16 by Toby.

1. A two-Cell Language Unit that spells a Number or a Note outside any operand slot is not diagnosed; every other spelling that names no Function still is. Operands a deleted or replaced Function leaves behind are no longer diagnosed. This answers ADR 0067's open question and needs an ADR (01) and a Parser change (04).
2. Language Map diagnostics are shown at the Cursor, as a status line in the bottom Panel (02).
3. Tick diagnostics are published by the Playback Engine and listed in a section of the Diagnostics Panel (03). The presentation is provisional and will be iterated once in use.
4. Language Map and Tick diagnostics are never shown in the same place, so `pending-presentation/04`'s duplicate report is not a prerequisite here; it stays a language ticket of its own.
5. The effort is not tagged `release/v1`.

## Out of scope

The rendering readouts in the Diagnostics Panel (`docked-diagnostics`), Playback failure messages (`console/src/diagnostics.rs`), marking Tick diagnostic anchors on the Grid, and a scrolling list of every diagnostic.
