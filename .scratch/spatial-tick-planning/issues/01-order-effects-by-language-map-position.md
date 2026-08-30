# 01 — Order effects by Language Map Position

**What to build:** Replace expression-only planning with ADR 0020's one row-major pass over all
actionable Language Units and Expression roots.

**Blocked by:** `.scratch/language-map/issues/03-move-source-consumers-behind-the-language-map.md`.

**Status:** ready-for-agent

- [ ] Turns are ordered by row-major anchor Position from one Source Snapshot.
- [ ] Each producer emits effects in a stable local order.
- [ ] Writes, activations, locks, diagnostics, and terminal commands share the same ordering model.
- [ ] Later effects win Cell conflicts independently after complete-write validation.
- [ ] Planned writes gain no same-Tick turn and generated Functions wait for the next Snapshot.
- [ ] A root whose turn passed is never revisited.
- [ ] Existing Tick Plan atomicity and Source/Playback seam remain intact.
