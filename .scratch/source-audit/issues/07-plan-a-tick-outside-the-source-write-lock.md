# 07 — Plan a Tick outside the Source write lock

**What to build:** Planning a Tick (scheduling, ordering, interpretation) no longer blocks the console's revision reads; only committing the planned writes takes the Source write lock. Today `SourceCommander::execute` takes the write lock and `Source::execute` plans and commits under that one guard on a Tokio worker. The pieces exist: `plan_tick(&self)` and `commit_tick(&mut self)` are already separate, `RevisionId` names a revision, and `SourceCommander::read_revision` already snapshots the Grid, bytes and an `Arc<LanguageMap>` under a read lock. That snapshot, `SourceRevision`, carries no `RevisionId`, so it cannot by itself validate a commit.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

Related: 06 shortens the commit's hold on the write lock (the Map rebuild) but is not a prerequisite. 19's schedule cache must be readable from where planning runs, so the two are designed together.

- [ ] Capture a consistent Source snapshot under a short read lock, then release all Source locks before planning. The snapshot records the `RevisionId` under the same read guard as the Grid, bytes and Language Map it holds, so the identity commit validates against is the one its contents came from. A planning-only snapshot type distinct from the public `SourceRevision` is acceptable. Revision validation and commit occur together under the write lock; planning does not hold a read guard that blocks editor writes.
- [ ] A commit rejects or re-plans when the revision it planned against is no longer current, so an edit made during planning is never overwritten.
- [ ] A rejected stale plan publishes no MIDI commands, stale diagnostics, or note-ownership changes. Only the accepted plan reaches Playback delivery.
- [ ] Record whether a retry keeps the same Tick and how repeated revision conflicts terminate or yield under continuous editing. Retries cannot duplicate effects or silently consume musical Ticks.
- [ ] Deterministic tests place an edit between plan and commit, reject a stale plan carrying MIDI effects, exercise repeated conflicts, and preserve ordering across consecutive accepted Ticks.
- [ ] A benchmark or profile measures both revision-read wait and editor-write latency under running Ticks, including contention and retry costs.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Still valid. The 06 blocker is weakened to a relation: planning can already run on a revision snapshot.

**2026-09-25 — acceptance-criteria review against `199c3331`.** Added stale-plan side-effect and retry contracts. A long-held read lock would still block editing; planning now explicitly uses an unlocked snapshot.

**2026-09-25 — snapshot identity required.** `SourceRevision` (`orcvs/src/source/mod.rs`) holds the Grid, bytes and Language Map but no `RevisionId`; the criterion now requires the planning snapshot to capture its identity consistently with its contents.
