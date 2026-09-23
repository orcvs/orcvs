# 12 — Dispose of the `feat/egui-theming` branch

**What to build:** Retire the local `feat/egui-theming` branch, now that `docs/research/egui-theming.md` is recovered from it byte-for-byte. Moved from `03`.

**Blocked by:** None — can start immediately.

**Status:** ready-for-human

- [ ] Decide whether the branch's history is worth keeping. If it is, tag its tip (for example `archive/feat-egui-theming`), and push the tag to `origin` only if the history should outlive this machine.
- [ ] Remove any worktree checked out on the branch (`git worktree list`) with `git worktree remove`.
- [ ] Delete the local branch.
- [ ] Record the tag name, or the decision not to tag, in a comment here.

## Comments

**2026-09-23 — opened by the audit of the merged pull requests against their issues.** `03`'s status said this needed "a human with push access to `orcvs/orcvs`". It does not: `git ls-remote` shows the branch on neither `origin` nor `fork`. It is a local branch with no upstream, last committed 2026-08-29. It needs a human because deleting unpushed history is not an agent's call.
