# 06 — Reuse unchanged Language Map rows across revisions

**What to build:** A Source revision reuses clean row derivations, so it does not deep-clone their Expressions. Today `LanguageMap::rebuild` calls `DerivedRow::for_revision` for every clean row, which deep-clones the row and its Expressions only to restamp the map identity. The shipped Grid is fixed at 256 rows (ADR 0054), and `Source::commit_tick` rebuilds on every Tick, including one that writes nothing, so every rebuild visits all 256 rows, derives written rows and deep-clones the clean ones.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] Unchanged rows are shared between consecutive revisions rather than cloned.
- [ ] Map identity is still checked wherever a row is resolved against its Map.
- [ ] Record the complexity and allocation target: clean Expression payloads are not copied, and any remaining O(rows) table traversal or shared-pointer copying is stated explicitly. Benchmarks through `test-grid-shapes` / `Grid::with_shape` measure scaling; `source_whole_grid/edit_rebuild_valid` measures the shipped-shape improvement. `source_file/read/256x256` is checked for regression.
- [ ] The allocation tests in `orcvs/tests/allocation.rs` that record the deep-clone finding reflect the new shape.
- [ ] Whether a commit with no writes mints a new Map revision is decided and recorded here. 19's cache key must survive no-write and identical-byte commits whichever way this is decided, so the decision does not supply that key.
- [ ] Language Map property tests still pass.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Still valid and more urgent: the fixed 256-row Grid makes every rebuild clone 256 rows. Benchmarks named; the no-write-commit decision added for 19.

**2026-09-25 — acceptance-criteria review against `199c3331`.** Sharing row payloads does not by itself remove height-dependent table work. The target now distinguishes deep-clone removal from the remaining traversal cost.
