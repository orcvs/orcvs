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

- [x] The configuration record and every entry point taking one are gone.
- [x] Scheduling and execution take no parameter that the shipped build discards.
- [x] The Tick scheduler and executor contain no `cfg` attribute on any field, signature,
      parameter, or statement of a shipped function. Test modules are unaffected.
- [x] The crate builds and its tests pass, and the workspace clippy and test gates are clean.
- [x] `docs/agents/` or `CLAUDE.md` records the standing rule: no shipped module compiles
      differently under test, and a test that needs an input production cannot produce constructs
      it below the shipped entry point.

## Comments

Split out of the architecture review of the Sequence branch, 2026-09-10. The rule in the last
acceptance line is a coding standard rather than a trade-off, so it belongs in the repository
contract and not in an ADR.

2026-09-10: Implemented. `Configuration`, `plan_configured`, `Source::execute_configured`, the
`configured_source` and `configured_tick` helpers and the `destinations` helper that folded a
Configuration are gone; `plan` now reaches `schedule` and `execution::execute` directly, and
`schedule`, `computations` and `execution::stated::plan_with_answers` each lost the parameter.

The `cfg` boundary, decided by the maintainer rather than read off the ticket. `carry`,
`schedule_carrying`, `plan_carrying` and `Source::execute_carrying` stay, `#[cfg(test)]` and all.
`07` left this open and the handoff for `08` named it as an ambiguity to resolve explicitly: these
four sit between a test module, which the third acceptance line exempts, and a shipped function
carrying a `cfg`, which it forbids. They are ruled test-only items, exempt. That line targets a
`cfg` attribute on a *shipped* function's field, signature, parameter or statement, and there are
now zero of those: every remaining `cfg` in `tick.rs` and `tick/execution.rs` sits on a whole
test-only item — `REFUSED_PORTAL`, `carry`, `schedule_carrying`, `plan_carrying`, `mod test`,
`mod stated` — or inside `mod test`.

The rule the fifth acceptance line asks for is recorded in `AGENTS.md` in sharpened wording, not
the line's own. "No shipped module compiles differently under test" is absolute, and this very
change contradicts it: `tick.rs` is a shipped module and gains four `cfg(test)` items, `model.rs`
one, and `playback.rs` already had six. Read literally the bullet would order the next agent to
delete exactly what the paragraph above deliberately keeps. The contract therefore states the rule
as behaviour — no shipped function takes a parameter, or reaches a branch, that only a test
populates — which is what this ticket actually enforced, and says the test-only item belongs beside
the shipped one rather than as a seam cut into it.

"Including the discard that silences the unused-parameter lint" was already satisfied. `03` deleted
`execute`'s `Configuration` parameter and the `let _ = configuration;` that silenced its lint in
`131a747`; nothing remained here to delete for that clause.

`REFUSED_PORTAL` is now raised in one place, `carry`, and so is `#[cfg(test)]` with a corrected doc
comment. After this change no production path gives a computation a *stated* destination at all:
`computations` resolves a root's ordinary result Portal below its own anchor and nothing else, and
hands a Terminal Output Function no destination to refuse. ADR 0009's Terminal-Output-Portal
refusal is therefore reachable only from a test-constructed carried schedule. That is the shape
`carry`'s own doc already predicted — ADR 0004's Source Function family, which would state such a
destination in Source, is unbuilt — but the refusal living entirely in `#[cfg(test)]` code is worth
naming rather than leaving to be discovered.

The coverage check. Nothing that survives covers `carry`'s `diagnostics.splice(0..0, refusals)`:
`live_inactive_ownership_and_terminal_portal_configuration_are_independent` drives the carried
refusal but earns one diagnostic, so it cannot witness an order. So
`a_refused_portal_is_diagnosed_before_the_row_edge_layout_it_shares_a_tick_with` was rewritten as a
carried-route-only test rather than deleted. Mutating the splice to an `extend` fails it.

`carry`'s closing assertion went with the route that could violate it, together with its
`#[should_panic]` test `carrying_destinations_onto_a_configured_refusal_is_refused`. With
`Configuration` gone, `computations` cannot hand `carry` a refusal to be ordered against.

`configured_tick`'s reasoning for the plan-only/state-reading helper split moved onto
`carried_tick`; `stated_tick` and `replaced_tick` now defer to `carried_tick`.
