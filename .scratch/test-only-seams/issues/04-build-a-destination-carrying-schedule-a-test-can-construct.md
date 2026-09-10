# 04 — Build a destination-carrying schedule a test can construct

**What to build:** The expand half of the destination migration. A test today gives a producer a
Portal destination other than its ordinary result position by populating a map the shipped
scheduler reads — a capability no production Tick has, because every Function that writes elsewhere
belongs to ADR 0004's unbuilt Source Function family. Provide a way for a test to assemble a
schedule carrying chosen destinations that is not an input the shipped planning path consults.

Nothing migrates in this ticket and nothing is deleted. Both routes exist when it lands, so every
existing test still passes untouched.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] A test can build a schedule whose producers carry chosen Portal destinations, and drive
      execution against it, without the shipped planning entry point taking a configuration.
- [ ] The new route produces the same schedule for the same inputs as the configured route it will
      replace, proven by at least one test asserted through both.
- [ ] The existing configured route is unchanged and every test still passes.

## Comments

Split out of the architecture review of the Sequence branch, 2026-09-10. Measured blast radius at
that date: roughly 52 call sites populate the destination map, against 22 for the answer map that
`01` through `03` retire. Expand–contract rather than one edit, so the tree stays green between
batches.
