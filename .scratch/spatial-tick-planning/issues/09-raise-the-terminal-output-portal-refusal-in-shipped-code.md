# 09 — Raise the Terminal Output Portal refusal in shipped code

**What to build:** Raise ADR 0009's "a Terminal Output Function cannot have a Portal" refusal in
shipped planning, and cover it with a test that is not the `#[cfg(test)]` `carry` helper.

**Blocked by:** None — no ticket yet owns input-named destinations for a Terminal Output Function.
Do not start until one does.

**Status:** ready-for-agent

Extracted from `spatial-tick-planning/03`. That ticket states destinations from Function
declarations. The first half of its inherited line is already true: stated destinations reach
`computations` without a Terminal Output Function acquiring one. The second half — a shipped
refusal when input *assigns* a destination to a Terminal Output Function — cannot fire until
Source can construct that pairing.

### Current hold

- `PortalAccess::resolve` (`orcvs/src/source/portal.rs`) returns `PortalWrites::None` when
  `function.performs_terminal_output()` is true. That gate runs before any destination arm.
- `stated_destinations_reach_computations_without_a_terminal_output_portal` covers a Self-Banging
  Function and Raw Play sharing one schedule: the former states a Portal, the latter acquires none,
  and no Portal diagnostic is raised.
- `carry` / `PortalAccess::carry` are still `#[cfg(test)]`. They no longer diagnose. A carried
  write onto Terminal Output leaves `PortalWrites::None` in place
  (`live_inactive_ownership_and_silent_terminal_output_are_independent`).

### Do not start until

A later Function (or addressing model) lets Source input name a destination for a Terminal Output
Function. Self-Banging, Directional Bang, Jump, and Halt all state Portals from declarations.
ADR 0005 still defers general Cell-address syntax. Inventing a test-only pairing so this ticket
can close is the seam `03` refused.

- [ ] Real input can assign a destination to a Terminal Output Function.
- [ ] ADR 0009's refusal is raised in shipped code on that pairing.
- [ ] A test of that shipped path covers the refusal. `carry` is not the raiser.
