# 08 — Refresh the inventory's Current state against the code

**What to build:** The "Current state" column of `01`'s shipped-language inventory states what the code does today, so `v1-release/03` can read it as the inventory of record. The definition of done requires this refresh before the inventory is used as authority.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Every inventory row's Current state is checked against the Function definitions in `lang` and the behaviour tests, and corrected. The column last refreshed on 2026-09-09 still calls the Self-Banging movement, the Sequence family, the Tick family and the Jump Functions missing; all of them are now defined.
- [ ] Every residual owner the column names is checked, and resolved owners are removed or replaced by what now owns the gap, if anything does.
- [ ] Each row links the tests that prove it, so `v1-release/03`'s inventory-to-evidence mapping can start from this table.
- [ ] Any member whose Current state is still short of the release is named with its owning `release/v1` issue, or recorded as an accepted deferral.
- [ ] The refresh date and the commit it was checked against are recorded.

## Comments

**2026-09-24 — opened by the release-membership audit.** `01` stays resolved: its decision stands and only its state column is stale. Blocks `v1-release/03`.
