# 01 — Settle the tracker interaction and operation contract

Status: needs-triage

## Problem

Selecting from a constructed Sequence can produce the right tune while losing
the requested interaction: editing notes directly in a row of Source Cells.
ADR 0017 names the intended composition but leaves its read operation undefined.

## Work

Record the smallest contract that supports a row of independently editable
notes, indexed by a Clock, with stable rest positions. Compare a region read
followed by Select with an indexed scalar read. State whether the read is
absolute or relative, the orientation supported initially, and what the reader
names: starting Position, step count, stride, or a declared region.

Reconcile ADR 0017's reserved Source-family names with ADR 0049's description of
absolute Address Functions. Do not choose a spelling by treating either prose
reference as an implemented signature. Preserve two-Cell Functions and typed
operands. Review the impact on ADRs 0005, 0007, 0017, 0019 and 0049.

## Acceptance

- [ ] A written walkthrough covers a note, a rest, wraparound, and a live edit.
- [ ] The chosen operation states its operands, result kind/width, activation,
      default Output Portal and supported data footprint.
- [ ] Step count is explicit; no implicit scanning to the first blank or stored
      hidden list supplies the melody. Decide zero count and index overflow.
- [ ] Editing a step never requires duplicating its note or constructing a
      singleton Range. Blank steps do not collapse the pattern.
- [ ] The decision identifies every ADR amendment and updates the glossary.
- [ ] Issues 02–08 are revised to the accepted contract before being marked
      ready for implementation; rejected alternative requirements are removed.

If a syntax prototype is used to resolve the decision, follow
`docs/agents/syntax-prototypes.md`: interactive Source Grids per Tick, explicit
hypothetical labels, diagnostics, deterministic walkthrough controls.
