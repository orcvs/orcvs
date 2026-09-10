# 08 — Delete the scheduler configuration and the parameter that carries it

**What to build:** The contract half. Delete the configuration record, the configured planning and
execution entry points, the test-only Source execution helper, and the parameter threaded through
scheduling and execution to carry it — including the discard that silences the unused-parameter
lint in the shipped build. Execution takes only what it uses.

Three tests still populate the destination map, and they are this ticket's to remove: their subject
is the configured route itself, so they go with it rather than migrating to the carried one. `07`
names them under "What `08` still owns". Every other caller had moved by the time `05`, `06`, `07`
and `10` resolved.

**Blocked by:** 03 — Delete the answer-substitution fork; 05, 06, 07 — the three destination
migration batches; 10 — Say what a Tick evaluated.

**Status:** ready-for-agent

- [ ] The configuration record and every entry point taking one are gone.
- [ ] Scheduling and execution take no parameter that the shipped build discards.
- [ ] The Tick scheduler and executor contain no `cfg` attribute on any field, signature,
      parameter, or statement of a shipped function. Test modules are unaffected.
- [ ] The crate builds and its tests pass, and the workspace clippy and test gates are clean.
- [ ] `docs/agents/` or `CLAUDE.md` records the standing rule: no shipped module compiles
      differently under test, and a test that needs an input production cannot produce constructs
      it below the shipped entry point.

## Comments

Split out of the architecture review of the Sequence branch, 2026-09-10. The rule in the last
acceptance line is a coding standard rather than a trade-off, so it belongs in the repository
contract and not in an ADR.
