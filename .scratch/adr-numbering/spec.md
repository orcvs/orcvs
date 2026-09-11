# Give every ADR one number and every citation one referent

**Status:** ready-for-agent

**Goal:** Two accepted ADRs are numbered 0036, and the repository cites ADRs by bare number in
prose. Every one of those citations outside `docs/adr/` is therefore ambiguous between two
decisions, and in `lang/src/atom.rs` adjacent comments resolve to different ones.

The decision this effort owes is which end to fix: the duplicate number, or the citations that
cannot carry a link. Issue 01 states both options and the evidence for each.
