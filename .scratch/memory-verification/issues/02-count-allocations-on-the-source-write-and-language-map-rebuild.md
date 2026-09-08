# 02 — Count allocations on the Source write and Language Map rebuild

**What to build:** Writing one Cell of the Source, and rebuilding the Language Map that derives from
it, are checkable for how much they allocate, so a per-Cell allocation on the editing path fails a
pull request.

**Blocked by:** 01 — Count allocations on the Tick and Render Frame paths.

**Status:** ready-for-agent

- [ ] Whether the counting allocator is duplicated in this crate's test binary or shared with
      `lang`'s is decided and the reasoning recorded. A test-only allocator does not go into a
      shipped crate's API to avoid forty duplicated lines.
- [ ] Writing one Cell of the Source is asserted against a shape: the in-place byte write should not
      allocate at all.
- [ ] A Language Map rebuild is asserted to be bounded independently of Grid size.
- [ ] Each assertion states whether it is measured on the calling thread, and any measurement that
      crosses a multi-threaded runtime uses atomics and says that it is correct only under nextest's
      process-per-test isolation.
- [ ] The tests run inside the existing `cargo nextest run --workspace --profile ci --locked` line.
      No mise task, no workflow change, and `scripts/check-tooling-contract.sh` is untouched.

## Comments

Two different paths share one ticket because they share one harness and one setup. If the Language
Map turns out to allocate per Cell, that is a finding and it belongs in this file — it is a result
about the Language Map, not a reason to split the ticket after the fact.

The Source write is the path that reaches the workspace's one `unsafe` block, which is also what
`06` scopes Miri to.
