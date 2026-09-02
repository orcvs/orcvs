# Order Tick planning and add performative spatial behavior

**Status:** ready-for-agent

## Goal

Implement ADRs 0004, 0006, 0009, 0014, and 0020 as one row-major Tick planning pass over the Language Map. Bang
activation, Self-Banging Function effects, Jump relay, Halt locks, ordinary writes, diagnostics,
and terminal commands share one total producer order.

## Delivery order

1. `issues/01-order-effects-by-language-map-position.md`
2. `issues/02-add-source-bang-activation-and-expiry.md`
3. `issues/03-add-directional-bang-movement.md`
4. `issues/04-add-directional-jump-chains.md`
5. `issues/05-add-halt-root-locking.md`

## Required behavior

- Every actionable Language Unit/root present in the Source Snapshot receives at most one turn.
- Producer anchor Position and emission order provide total deterministic effect ordering.
- Activation can affect only a root whose turn has not passed; roots are never revisited.
- Every complete write validates before entering the Tick Plan; conflicts resolve Cell-wise.
- Planned writes never become executable Source during the same Tick.

## Out of scope

- General Cell-address syntax deferred by ADR 0005.
- Sequence transport through Jump.
- Additional control phases or hidden activation queues outside the Tick Plan.
