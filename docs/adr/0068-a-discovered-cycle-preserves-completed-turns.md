# A discovered cycle preserves completed Turns

Status: accepted. Amends the cycle-stopping guarantee in [ADR 0065](0065-an-error-never-stops-the-performance.md) for dependencies discovered during a Tick under [ADR 0067](0067-track-reads-a-portal-as-jump-does.md).

A cycle known before execution stops every computation in the Expressions it
reaches. A cycle discovered at a Turn stops every remaining computation in those
Expressions and their dependants, including nested siblings whose own inputs
are ready. Turns already completed retain their Effects, including writes from
a nested Function that supplied Track's index or count. The cycle is diagnosed
under ADR 0065's representative and downstream-waiting rules, and independent
Expressions continue.

Track cannot identify its Input Portal until its operands settle. Those operands
can themselves write Source through their Output Portals, and later completed
Turns can consume those writes. Discarding just an earlier write would leave its
consequences standing; rolling back all consequences would require tracking or
replanning execution. We preserve completed Turns rather than introduce that
rollback. No remaining Turn in an affected Expression executes after the cycle
is discovered, and nothing is delayed to a later Tick.
