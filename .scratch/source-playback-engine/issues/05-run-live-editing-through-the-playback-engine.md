# 05 — Run Live Editing through the Playback Engine

**What to build:** Introduce the Playback Engine described by ADR-0001 so Playback timing drives fresh Source Tick Plans while the engine dispatches each committed Play Command list exactly as supplied through an in-memory output adapter.

**Blocked by:** 04 — Interpret terminal Play Functions into Play Commands.

**Status:** resolved

- [x] A Playback clock Tick causes orchestration to request one fresh Source Tick Plan, commit its Source writes, and only then submit its Play Commands.
- [x] The Playback Engine never parses Source, interprets Expressions, infers sustain, removes repeated commands, or invents note-off timing.
- [x] Live Editing affects the next Tick whose Source snapshot has not yet been taken.
- [x] Only one Tick transition runs at a time; missed deadlines are dropped, Playback resumes on the next scheduled Tick, and an overrun diagnostic is observable.
- [x] The in-memory adapter receives one ordered list per successful Tick and can verify exact repeated commands.
- [x] Output-adapter failure is diagnosed without stopping Playback or rolling back committed Source writes.
- [x] Stopping Playback or disconnecting the adapter sends all-notes-off as the sole transport-safety exception.
- [x] Playback timing and dispatch behavior are testable without sleeping in wall-clock time.

## Answer

The Playback Engine owns lifecycle and scheduled musical time, requests and commits one fresh Tick Plan per accepted Tick, then dispatches its exact ordered Play Command list. Live Editing, overruns, restart safety, adapter failures, and all-notes-off behavior are covered with deterministic adapters and paused time.

## Comments

**2026-08-26 — implemented (agent)**

Playback lifecycle and scheduled Tick orchestration now live in a Playback Engine. Each accepted clock Tick synchronously obtains and commits one fresh Tick Plan before the output adapter receives its exact Play Command list. A deterministic in-memory adapter and explicit scheduled/observed times cover Live Editing, repeated dispatch, overruns, output failure, and all-notes-off safety without wall-clock sleeps.
