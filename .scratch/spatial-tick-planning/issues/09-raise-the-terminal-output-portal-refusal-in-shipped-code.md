# 09 — Raise the Terminal Output Portal refusal in shipped code

**What to build:** Raise the "a Terminal Output Function cannot have a Portal" refusal in shipped
planning (no ADR states it; `CONTEXT.md`'s **Output Portal** entry says a Terminal Output Function
writes nothing there), and cover it with a test that is not the `#[cfg(test)]` `carry` helper.

**Blocked by:** None — no ticket yet owns input-named destinations for a Terminal Output Function.
Do not start until one does.

**Status:** needs-triage

Extracted from `spatial-tick-planning/03`. That ticket states destinations from Function
declarations. The first half of its inherited line is already true: stated destinations reach
`computations` without a Terminal Output Function acquiring one. The second half — a shipped
refusal when input *assigns* a destination to a Terminal Output Function — cannot fire until
Source can construct that pairing.

### Current hold

- `PortalAccess::resolve` (`orcvs/src/source/portal.rs`) returns `PortalOutput::None` when
  `function.performs_terminal_output()` is true. That gate runs before any destination arm.
- `stated_destinations_reach_computations_without_a_terminal_output_portal` covers a Self-Banging
  Function and Raw Play sharing one schedule: the former states a Portal, the latter acquires none,
  and no Portal diagnostic is raised.
- `carry` / `PortalAccess::carry` are still `#[cfg(test)]`. They no longer diagnose. A carried
  write onto Terminal Output leaves `PortalOutput::None` in place
  (`live_inactive_ownership_and_silent_terminal_output_are_independent`).

### Do not start until

A later Function (or addressing model) lets Source input name a destination for a Terminal Output
Function. Self-Banging, Directional Bang, Jump, and Halt all state Portals from declarations.
ADR 0049 superseded ADR 0005's addressing deferral and names the absolute `&` Address Functions
as follow-up work; no ticket owns them yet. Inventing a test-only pairing so this ticket
can close is the seam `03` refused.

- [ ] Real input can assign a destination to a Terminal Output Function.
- [ ] The refusal is raised in shipped code on that pairing.
- [ ] A test of that shipped path covers the refusal. `carry` is not the raiser.

## Comments

### Audit at cad296df — 2026-09-29

- Status `ready-for-agent` → `needs-triage`. The ticket's own "Do not start until" section says it
  waits for input that can assign a Terminal Output destination. No such input exists, and no
  ticket owns it.
- `PortalWrites` → `PortalOutput`. The gate in `PortalAccess::resolve` returns
  `PortalOutput::None` (`orcvs/src/source/portal.rs:484,490`); `PortalWrites` no longer exists.
- ADR 0009 never mentions Terminal Output, and no ADR states this refusal. The nearest statement is
  the **Output Portal** entry in `CONTEXT.md`. ADR 0028 and 0016 cover only where Terminal Output
  Functions may appear, not Portals.
- ADR 0005's addressing deferral is superseded by ADR 0049, which fixes the address space and
  leaves absolute `&` Address Functions as follow-up work. Such a Function is the likely owner of
  the first criterion.
