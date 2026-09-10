# 08 — Delete the scheduler configuration and the parameter that carries it

**What to build:** The contract half. With no caller populating either map, delete the
configuration record, the configured planning and execution entry points, the test-only Source
execution helper, and the parameter threaded through scheduling and execution to carry it —
including the discard that silences the unused-parameter lint in the shipped build. Execution takes
only what it uses.

**Blocked by:** 03 — Delete the answer-substitution fork; 05, 06, 07 — the three destination
migration batches.

**Status:** ready-for-agent

- [ ] The configuration record and every entry point taking one are gone.
- [ ] Scheduling and execution take no parameter that the shipped build discards.
- [ ] The Tick scheduler and executor contain no `cfg` attribute that changes what the shipped
      module computes.
- [ ] The crate builds and its tests pass, and the workspace clippy and test gates are clean.
- [ ] `docs/agents/` or `CLAUDE.md` records the standing rule: no shipped module compiles
      differently under test, and a test that needs an input production cannot produce constructs
      it below the shipped entry point.

## Comments

Split out of the architecture review of the Sequence branch, 2026-09-10. The rule in the last
acceptance line is a coding standard rather than a trade-off, so it belongs in the repository
contract and not in an ADR.
