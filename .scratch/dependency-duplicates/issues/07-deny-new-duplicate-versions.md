# 07 — Deny new duplicate versions

**What to build:** A dependency change that introduces a second version of a crate fails the audit at the pull request that brings it, rather than joining a list of warnings nobody reads. With every existing duplicate either unified (04) or recorded with its reason (01, 05), the skip list is the complete set of known duplicates, and `[bans] multiple-versions` becomes `"deny"`.

A routine `cargo update` must not fail the gate. It can move a skipped crate to a new patch release, or re-raise the `windows-sys` version that 04 unified. Skip entries match the held release line, not one exact patch version, where cargo-deny's package spec allows it. `docs/tooling.md` says how to handle a failing duplicate: unify it if the dependents allow it, otherwise add a skip entry that names the upstream crate holding it.

**Blocked by:** 04 — Unify arboard's windows-sys on 0.52; 05 — Skip the duplicates upstream releases still hold.

**Status:** resolved

- [x] `[bans] multiple-versions` is `"deny"`, and `mise run audit_deps` passes.
- [x] Every skip entry, including the ones from 01, still matches after a patch release of the crate it names.
- [x] `docs/tooling.md` records how to resolve a duplicate-version failure.
