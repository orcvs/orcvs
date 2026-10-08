# 02 — Present a pending Function as unfinished Source

Status: needs-triage
Blocked by: 01

**What to decide and build:**

ADR 0069 makes every unwritten slot a diagnostic, including each rest on which a Bang reaches a Play. The console has to present a pending diagnostic as unfinished Source, not as a fault, or the change costs the performer a noisy Diagnostics Panel.

## Questions

- How Source Paint marks a pending Function and its unwritten slot, and whether it names the type the slot waits for.
- Whether the Diagnostics Panel lists pending diagnostics, groups them apart from faults, or leaves them out.
- Whether a rest, a Play left pending by a Bang, is presented at all while Playback runs.

## Acceptance criteria

- [ ] The answers above are recorded under `## Answer` here and, where they are vocabulary, in `CONTEXT.md`.
- [ ] Source Paint and the Diagnostics Panel present pending diagnostics as decided, tested through the real App under the egui skill.
