# Derive one Language Map from each Source revision

**Status:** ready-for-agent

## Goal

Implement ADR 0018 by replacing the parallel Expression extent, parsed Atom, Glyph, and diagnostic
views with one deep Language Map module derived from character Source. The Source remains the only
stored program state and the Grid remains the only coordinate system.

## Required behavior

- Partition each row left-to-right into non-overlapping complete Language Units.
- Give every valid Language Unit one anchor Position and complete Footprint.
- Resume after a complete Footprint, or after one invalid character when recognition fails.
- Derive Expressions and roots without wrapping across Grid rows.
- Keep incomplete and invalid Live Edits as valid character Source with diagnostics.
- Supply parsing, interpretation, spatial behavior, Glyph classification, and diagnostics through
  the Language Map interface rather than parallel caller-owned arrays or scans.
- Preserve Source persistence by storing only Grid and character Cells, then rebuilding the map.

## Delivery order

1. `issues/01-partition-source-into-language-units.md`
2. `issues/02-derive-expressions-roots-and-diagnostics.md`
3. `issues/03-move-source-consumers-behind-the-language-map.md`

## Out of scope

- Sequence evaluation, spatial activation, and new Function semantics.
- Sparse or infinite Source storage; finite Grid assumptions stay behind the module seam.
- Source address syntax deferred by ADR 0005.

## Decisions

ADRs 0018 and 0020 are authoritative. `CONTEXT.md` defines Language Unit, Footprint, Language Map,
Expression, Position, and Grid.
