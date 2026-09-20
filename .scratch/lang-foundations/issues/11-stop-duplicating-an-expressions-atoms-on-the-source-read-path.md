# 11 — Stop duplicating an Expression's Atoms on the Source read path

**What to build:** Read an Expression's parsed values without building a fresh collection on every read, and stop storing a second copy of those values beside the Expression that already holds them.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

**Tags:** Improvement

Every read of an Expression's values allocates a new collection, and the Source read path performs that read per Expression, per call, at Render Frame rate. The Language Map then caches the result beside the Expression it came from, so a populated Source holds each Expression's values twice.

- [ ] Reading an Expression's values does not allocate per call on the Source read path.
- [ ] The Language Map holds one representation of an Expression's values rather than an Expression and a duplicate of its values.
- [ ] The existing allocation-shape assertions cover the read path and pass. A shape they reveal is recorded before it is relaxed, as that suite requires.
- [ ] Interpretation results, Language Map behavior, and native and WASM callers are unchanged.

## Comments

This is the part of the original `lang` artifact's redundant-reconstruction finding that survived auditing. The rest of that finding described a bounded-buffer handoff inside `lang` that issue 07 removed; see issue 04's comments. The duplication that remains is in `orcvs`, on the path the allocation suite was written to watch, which is why that suite is the gate here rather than the benchmark series — allocation shapes answer this question and wall-clock on a shared runner does not.
