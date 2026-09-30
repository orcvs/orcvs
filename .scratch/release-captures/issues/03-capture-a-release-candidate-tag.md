# 03: Capture a release-candidate tag

**What to build:** Run the release captures when a release-candidate tag is pushed, so a tagged candidate gets its images without anyone dispatching the workflow (user story 11). The repository has no candidate tag convention yet; choosing one is the first step. Until `02` lands, the workflow has only the native job, so a tag run produces the four native images and no web captures. See `../spec.md`.

**Blocked by:** 01 — Capture the native console in CI.

**Status:** ready-for-human

- [ ] A release-candidate tag convention is chosen and recorded in `docs/tooling.md`, precise enough for a `push: tags:` filter to match candidates and nothing else.
- [ ] `.github/workflows/release-captures.yml` gains a `push: tags:` trigger on that pattern beside `workflow_dispatch`. A tag push captures the tagged commit; the SHA input still drives a dispatch.
- [ ] `scripts/check-tooling-contract.sh` allows exactly that tag trigger beside `workflow_dispatch` on a workflow that runs the capture, and still rejects a pull request, a push to a branch, a merge group and a schedule. `scripts/tests/check-tooling-contract.sh` rejects a tag pattern wider than the convention.
- [ ] The manifest records the tag when a tag push produced the run, and `docs/tooling.md`'s capture section describes the tag trigger.
- [ ] One real tag push produces the artifact, and its run link is recorded here.
- [ ] `actionlint`, `zizmor --offline .github/workflows`, `bash scripts/check-tooling-contract.sh` and `bash scripts/tests/check-tooling-contract.sh` pass.

## Comments

**2026-09-29 — split from 01.** 01 shipped with manual dispatch alone because there was no candidate tag convention to match. Until this lands, a tagged candidate is captured by dispatching **Release captures** with its SHA.

**2026-09-30 — audit.** Checked against tracker conventions: `Status: ready-for-human` (the tag convention is a human choice) and no `Tags: release/v1`, which is correct because this ticket is outside the release Gate's dependency closure and a tagged candidate can be captured by dispatch. Added the `docs/tooling.md` capture-section clause to the manifest criterion, since that section describes the workflow as `workflow_dispatch` alone, and stated in the body that a tag run is native-only until `02` lands.
