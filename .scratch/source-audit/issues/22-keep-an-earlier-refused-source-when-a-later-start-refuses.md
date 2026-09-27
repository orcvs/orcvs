# 22 — Keep an earlier refused Source when a later start also refuses

**What to build:** A stored value that fails to decode is set aside so the viewer can recover it, and a second refusing start does not overwrite what the first set aside. Today saving the refused payload writes its key unconditionally, so two refusing starts in a row lose the first payload.

**Blocked by:** None — can start immediately.

**Status:** wontfix

- [ ] A second refusal does not replace an existing set-aside payload (it is kept, or both are kept under distinct keys).
- [ ] A test performs two refusing starts in a row and recovers the first payload.
- [ ] The feature-off build still compiles without persistence.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Still valid; no change needed.

**2026-09-27 — wontfix by maintainer decision; superseded by [#170](https://github.com/orcvs/orcvs/pull/170).** A stored Source that does not decode is discarded rather than set aside: the console reports one error, starts the empty Grid, and its next save overwrites `orcvs_source`. `orcvs_source_refused` and the persistence notice are removed, so there is no earlier payload for a later refusal to replace. Reason: Orcvs is pre-release with no compatibility contract, and the set-aside only protected a rare cross-build case. PR #167, which kept every refused payload under numbered keys, was closed as over-engineered. The criteria above are not met and no longer apply.
