# 10 — Reconcile the tick-execution-order tickets with the corrected partition

**What to build:** Bring the settled `tick-execution-order` tickets into line with the code the
`cell-indexed-parse` effort leaves behind. Those tickets are `resolved`, and they describe a
scheduler that reads a whitespace partition and rebuilds an Expression's Cell positions for itself.
Neither is true afterwards.

**Blocked by:** 09 — Decide what an incomplete Function claims; cell-indexed-parse/05 — Delete the
reconstruction.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] `tick-execution-order/01`, `02` and `03` are re-stated or superseded. They are `resolved` but
      describe code the `cell-indexed-parse` effort rewrites.
- [ ] ADR 0031 and ADR 0032 are unchanged. They decide ordering, not partition, and this issue
      confirms rather than edits them.
- [ ] The Tick behaviour the corrected partition changes is stated where it is tested, not twice:
      `.+0102.+0304` as two roots and `.+0102Z` as an executing Expression belong to
      `cell-indexed-parse/07`, and this issue does not restate them.

## The rebase plan is withdrawn

This issue was written as a plan to rebuild branch `06-optimize-tick-scheduling` so that the
scheduler was never written against the broken partition. It named two commits to drop, `4d6c17c`
and `8e7bdce`, because both state the whitespace run in their own messages.

**That plan is no longer the plan, and its own goal is already spent.** The compensations exist. They
were written, reviewed, and corrected: the join guard now searches every Expression run rather than
only scheduled roots, and it reads the runs as the Tick leaves them. A rebuild cannot un-write them,
and it would discard the review that checked them.

What replaces it:

- `cell-indexed-parse/01` corrects the partition in place. Nothing is rebased onto it.
- `cell-indexed-parse/04` deletes the join guard at the point the parse boundary makes it dead. It
  cannot go earlier, because until `01` lands the guard is the only thing preventing a real
  corruption.
- `cell-indexed-parse/05` deletes the layout reconstruction, so the structural rules this issue
  worried about have nothing left to compensate for.
- `cell-indexed-parse/07` carries the language changes and the worked examples this issue listed.

The one thing no other ticket carries is the first checkbox above, so that is what this issue is now.

## Comments

Reduced to its residue after the `cell-indexed-parse` effort took over the partition work.

The original rebase plan is not in this file's history, because the file had never been committed
when it was rewritten. Its substance is summarised above: land the partition work first, rebase the
scheduler commits onto it, and drop `4d6c17c` and `8e7bdce` because both state the whitespace run in
their own messages. Its commit-by-commit rebase analysis of `31d1b1a` is gone.
