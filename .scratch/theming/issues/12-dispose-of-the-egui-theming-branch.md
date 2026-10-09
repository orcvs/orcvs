# 12 — Dispose of the `feat/egui-theming` branch

**What to build:** Retire the local `feat/egui-theming` branch, now that `docs/research/egui-theming.md` is recovered from it byte-for-byte. Moved from `03`.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Decide whether the branch's history is worth keeping. *(Not kept: discarded by the user on 2026-10-09.)* If it is, tag its tip (for example `archive/feat-egui-theming`), and push the tag to `origin` only if the history should outlive this machine.
- [x] Remove any worktree checked out on the branch (`git worktree list`) with `git worktree remove`. None remains (checked 2026-09-29).
- [x] Delete the local branch.
- [x] Record the tag name, or the decision not to tag, in a comment here.

## Comments

**2026-09-23 — opened by the audit of the merged pull requests against their issues.** `03`'s status said this needed "a human with push access to `orcvs/orcvs`". It does not: `git ls-remote` shows the branch on neither `origin` nor `fork`. It is a local branch with no upstream, last committed 2026-08-29. It needs a human because deleting unpushed history is not an agent's call.

**2026-09-29 — audit at `cad296df`.** The branch is still local only, tip `2eed1147` (2026-08-29), 58 commits ahead of `main`, none of them patch-identical on `main` (`git cherry`). No worktree is checked out on it — `git worktree list` shows only `main`, `.worktrees/safe-source-storage` and `.worktrees/source-file-bench` — so the worktree line is ticked. `docs/research/egui-theming.md` matches the branch copy byte-for-byte. One knock-on for the delete: `console/src/theme.md:196-197` cites "the hand-tuned `LIGHT_PALETTE` on the `feat/egui-theming` branch", which exists only there. Deleting without a tag leaves that citation dangling, so either tag the tip and name the tag in `theme.md`, or reword `theme.md` in the same change. No tag exists yet (`git tag -l '*theming*'` is empty).

**2026-10-09 — resolved: not tagged.** By the audit at `d33abd5f` the local branch was already deleted, without a tag; no ref contained its tip `2eed1147`, which survived only as an unreachable object. The user decided to discard the history rather than tag it, so the tip goes at the next `git gc`. `docs/research/egui-theming.md` is the one file recovered from it. `console/src/theme.md`'s Orcvs Light section no longer cites the branch: it cites the `LIGHT_PALETTE` values this repo keeps in `04`.
