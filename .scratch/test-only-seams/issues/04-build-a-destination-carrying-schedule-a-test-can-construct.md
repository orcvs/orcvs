# 04 — Build a destination-carrying schedule a test can construct

**What to build:** The expand half of the destination migration. A test today gives a producer a
Portal destination other than its ordinary result position by populating a map the shipped
scheduler reads — a capability no production Tick has, because every Function that writes elsewhere
belongs to ADR 0004's unbuilt Source Function family. Provide a way for a test to assemble a
schedule carrying chosen destinations that is not an input the shipped planning path consults.

Nothing migrates in this ticket and nothing is deleted. Both routes exist when it lands, so every
existing test still passes untouched.

**Blocked by:** None (can start immediately).

**Status:** resolved

- [x] A test can build a schedule whose producers carry chosen Portal destinations, and drive
      execution against it, without the shipped planning entry point taking a configuration.
- [x] The new route produces the same schedule for the same inputs as the configured route it will
      replace, proven by at least one test asserted through both.
- [x] The existing configured route is unchanged and every test still passes.

## Comments

Split out of the architecture review of the Sequence branch, 2026-09-10. Measured blast radius at
that date: roughly 52 call sites populate the destination map, against 22 for the answer map that
`01` through `03` retire. Expand–contract rather than one edit, so the tree stays green between
batches.

2026-09-10: Resolved by the commit this line lands in. No shipped code changed: `#53` had already
split `schedule` into `computations` and `order_turns`, which is exactly the seam this needed, so
the whole change is `cfg(test)`.

`carry` gives stated computations chosen destinations after `computations` returns them, applying
the same Terminal Output gate `computations` applies. `schedule_carrying`, `plan_carrying` and
`Source::execute_carrying` drive a Tick against the result. The test helper `carried_source` takes
the arguments `configured_source` takes, so migrating a test in `05` through `07` is a change of
helper and nothing else.

`carry` raises its refused-Portal diagnostics at the front of the diagnostics `computations`
returned, not the back. `computations` has two things to say and says the Portal refusal while it
resolves destinations, before the walk that raises the layout diagnostics; under the empty
Configuration a carried schedule states, everything it returns is a layout diagnostic.
`a_refused_portal_is_diagnosed_before_the_row_edge_layout_it_shares_a_tick_with` pins that order
against the configured route so the placement is covered rather than assumed.

One route is not carried yet: `execution::stated::plan_with_answers` still takes a `Configuration`,
so the `stated_source` and `sequence_source` fixtures still state their destinations through one.
That function is already wholly `cfg(test)` and already calls `computations` directly, so migrating
it is a matter of swapping its parameter and calling `carry` — `05` through `07` work, not a route
`04` owes.
