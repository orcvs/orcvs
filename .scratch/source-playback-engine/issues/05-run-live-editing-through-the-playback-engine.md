# 05 — Run Live Editing through the Playback Engine

**What to build:** Introduce the Playback Engine described by ADR-0001 so Playback timing drives fresh Source Tick Plans while the engine dispatches each committed Play Command list exactly as supplied through an in-memory output adapter.

**Blocked by:** 04 — Interpret terminal Play Functions into Play Commands.

**Status:** ready-for-agent

- [ ] A Playback clock Tick causes orchestration to request one fresh Source Tick Plan, commit its Source writes, and only then submit its Play Commands.
- [ ] The Playback Engine never parses Source, interprets Expressions, infers sustain, removes repeated commands, or invents note-off timing.
- [ ] Live Editing affects the next Tick whose Source snapshot has not yet been taken.
- [ ] Only one Tick transition runs at a time; missed deadlines are dropped, Playback resumes on the next scheduled Tick, and an overrun diagnostic is observable.
- [ ] The in-memory adapter receives one ordered list per successful Tick and can verify exact repeated commands.
- [ ] Output-adapter failure is diagnosed without stopping Playback or rolling back committed Source writes.
- [ ] Stopping Playback or disconnecting the adapter sends all-notes-off as the sole transport-safety exception.
- [ ] Playback timing and dispatch behavior are testable without sleeping in wall-clock time.
