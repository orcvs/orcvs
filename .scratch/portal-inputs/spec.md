# Portal inputs for feedback Functions

**Status:** resolved

## Goal

Re-thread Increment `~+` and Interpolation `~>` onto Portal input binding — the model ADRs
0012, 0004, 0009, and 0024 already describe — instead of smuggling Portal content through
`TickInputs.previous`.

Portal inputs are Function inputs resolved at Turn from working Source, symmetric to Portal
outputs. Feedback at the ordinary result Portal is the degenerate case: input Portal and output
Portal are the same site.

## Delivery order

1. `issues/01-clarify-adr-0012-feedback-as-portal-reads.md`
2. `issues/02-declare-portal-inputs-on-increment-and-interpolation.md`
3. `issues/03-bind-portal-inputs-at-turn-for-increment-and-interpolation.md`
4. `issues/04-retire-previous-from-tickinputs.md`

## Required behavior

- Increment and Interpolation read one Number from the ordinary result Portal in working Source at
  Turn, treating empty Cells as Number `00` and diagnosing any other present Language Unit.
- Portal inputs bind through the same validation contract as cell operands; operand faults precede
  Portal faults when both are wrong.
- Cross-Tick state remains visible only in Source Snapshot Cells — nothing tracked as hidden
  interpreter state.
- All behavioural acceptance from `tick-functions/03` continues to hold after the re-thread.

## Out of scope

- General `@<` Source Read with declared portal geometry (ADR 0008/0017; operands deferred per
  ADR 0005).
- Converging Delay and Euclidean onto a typed Tick binding seam (`ScalarAtTick`).
- Random anchor threading (remains `tick-functions/04`).
