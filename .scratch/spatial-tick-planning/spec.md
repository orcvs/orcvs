# Order Tick planning and add performative spatial behavior

**Status:** ready-for-agent

## Goal

Integrate performative spatial behavior with the dependency-ordered Tick contract of ADR 0032. The replacement execution foundation is tracked in [tick-execution-order](../tick-execution-order/spec.md). Directional movement, Jump, and Halt must adapt their read/write and activation semantics to that contract before implementation; their old row-major assumptions are not authorization to restore the superseded runtime.

## Delivery order

1. `issues/01-order-effects-by-language-map-position.md`
2. `issues/02-add-source-bang-activation-and-expiry.md`
3. `issues/03-add-the-self-banging-functions.md`
4. `issues/06-add-the-directional-bang-functions.md`
5. `issues/04-add-directional-jump-chains.md`
6. `issues/05-add-halt-root-locking.md`

## Historical required behavior — superseded by ADR 0032

- Every actionable Language Unit/root present in the Source Snapshot receives at most one turn.
- Producer anchor Position and emission order provide total deterministic effect ordering.
- Activation can affect only a root whose turn has not passed; roots are never revisited.
- Every complete write validates before entering the Tick Plan; conflicts resolve Cell-wise.
- Planned writes never become executable Source during the same Tick.

## Out of scope

- General Cell-address syntax deferred by ADR 0005.
- Sequence transport through Jump.
- Additional control phases or hidden activation queues outside the Tick Plan.
