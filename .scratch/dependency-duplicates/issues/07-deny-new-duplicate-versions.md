# 07 — Deny new duplicate versions

**What to build:** A dependency change that introduces a second version of a crate fails the audit at the pull request that brings it, rather than joining a list of warnings nobody reads. With every existing duplicate recorded with the upstream crate that holds it (01, 05), the skip list is the complete set of known duplicates, and `[bans] multiple-versions` becomes `"deny"`.

The gate fails on a duplicate that is new, and never on a known one moving within its release line. A new duplicate brought in by a routine `cargo update` or a Dependabot bump fails that pull request on purpose, and the fix is a decision: unify it through a manifest requirement, or add a skip entry naming the upstream crate that holds it. A patch release of an already-skipped crate carries no decision, so skip entries match the held release line (`syn@2`), not one exact version, which is the only thing a full version after `@` matches. The skip list describes the graph the resolver produces; it never depends on a lockfile state that `cargo update` would undo. `docs/tooling.md` says how to handle a failing duplicate.

**Blocked by:** 05 — Skip the duplicates upstream releases still hold.

**Status:** resolved

- [x] `[bans] multiple-versions` is `"deny"`, and `mise run audit_deps` passes.
- [x] Every skip entry, including the ones from 01, names a release line, so a patch release of the crate it names still matches.
- [x] After a real `cargo update`, the audit fails only on duplicates the skip list does not know (checked 2026-09-25: only `miniz_oxide` 0.9, which `flate2` 1.1.10 newly brings in).
- [x] `docs/tooling.md` records how to resolve a duplicate-version failure.
