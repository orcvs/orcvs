# 04: Delete the join guard the parse boundary makes dead

**What to build:** Remove the guard that refuses a result written flush against a neighbouring
Expression, because once the boundary comes from arity rather than from spaces there is nothing left
for it to prevent.

The guard exists because a row is currently partitioned at its spaces, so a two-Cell result written
flush against another Expression joins the two into one longer run, and the next parse reads a
different Expression than the one that was there. Under partition-by-parse the next parse resumes by
arity from each anchor. A result written in the Cells before a neighbour leaves that neighbour's
anchor and extent exactly where they were, so abutment changes nothing. Only overlap can, and
overlap is no longer a special case: the array says the Cells belong to another Expression, and the
classification already refuses a structural write and diagnoses a partial one.

`language-map/10` reached this first and states it as the reason to drop the commit that introduced
the guard rather than to rewrite it. This ticket is that deletion, done in place rather than by
rebase, so the two do not specify opposite things.

**The guard currently works and its tests pass.** It was fixed on this branch after review: it now
searches every Expression run rather than only scheduled roots, and it reads the runs as the Tick
leaves them so a producer rewriting its own Bang display joins nothing. Deleting it must not
resurrect the corruption it was written for — the point is that the parse boundary prevents that
corruption at its source, so the same Source must stay safe with the guard gone.

**Blocked by:** 03

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] A comparison writing its result flush against a literal leaves that literal readable, and the
      next parse establishes the same Expression it did before the write.
- [ ] A result written flush against an Expression holding a Function candidate leaves that
      Expression's anchor and extent unchanged.
- [ ] A result overlapping another Expression's Cells is still refused, by the classification rather
      than by a guard of its own.
- [ ] The adjacency search over Expression Spans, the surviving-runs extension it needed, and the
      diagnostic they produced are deleted.
- [ ] The join tests are replaced by tests that the same Source is now safe rather than diagnosed,
      and no test is deleted without its replacement naming the same Source.
- [ ] `language-map/10` no longer needs to drop a commit this ticket has already removed.
