# 01 — Prototype the Activation representation

**What to build:** Implement throwaway, focused alternatives for Activation recognition and one-Tick
planning so the production ticket can choose between a distinct spatial Language Unit and a
self-reproducing Function/value model from evidence rather than ADR shape alone.

**Blocked by:** language-map/01 — Partition Source into Language Units.

**Status:** resolved

**Tags:** release/v1

- [x] Both alternatives recognize all four encodings and preserve direction in Source.
- [x] Both plan exactly one move per Tick and replace the current Footprint with Bang on collision or
      an out-of-Grid move.
- [x] The comparison exercises initial emission, overlapping movement, aligned root contact,
      partial-unit contact, generated Source, and Source-order scheduling.
- [x] The comparison states what Function evaluation, runtime-value, Sequence, Source-write, and
      Glyph machinery each alternative reuses or duplicates.
- [x] The chosen model maximizes locality behind the Language Map/Tick-planning interface and does
      not grant unrequested Expression or Sequence capabilities.
- [x] The discarded prototype code is removed; the answer and selected representation constraints
      are recorded in this ticket before `spatial-tick-planning/03` begins.

## Answer

Choose a distinct spatial `LanguageUnitKind::Activation(Direction)` and schedule it directly from
the Language Map. `Direction` may remain a small shared encoding type, but an Activation Character
must not be an `Atom`, `Function`, ordinary Expression value, or Sequence member.

The focused prototype ran both candidates through the same Source-grid scenarios. Both recognized
`^^`, `vv`, `<<`, and `>>` with their direction intact; emitted the matching encoding from an active
Directional Bang Function; planned exactly one move for each Activation Character present in the
Source Snapshot; allowed horizontal self-overlap by ordering old-Footprint clears before new writes;
changed the current Footprint to `**` on collision or an out-of-Grid move; delivered activation only
for complete aligned root contact; diagnosed partial-unit contact without delivery; deferred
generated Source until the following Tick; and activated a contacted root only when its row-major
turn was still ahead. The observable behavior did not distinguish the candidates.

The retained guided evidence is
[`lang/activation-representation-prototype.html`](../../../lang/activation-representation-prototype.html).

### Comparison

| Concern | Distinct spatial Language Unit | Self-reproducing Function/source value |
| --- | --- | --- |
| Recognition and direction | Reuses the Language Map's left-to-right partition, anchor, two-Cell Footprint, and `Activation(Direction)` unit kind. | Reuses `Atom::Activation`, parser records, `Display`, and the standalone-Activation Expression path. |
| One-Tick scheduling | Reuses the row-major Language Unit/Expression-root pass. Snapshot units receive one turn; planned writes receive none. | Requires a special case to schedule an Activation-only Expression because ordinary evaluation schedules Functions, not inert values. A Function form would also require bypassing normal activation gating. |
| Movement and collision | Reuses Grid positions, Footprints, Language Map occupancy/root queries, diagnostics, and ordered Tick-plan Cell writes. | Duplicates the same spatial planner: ordinary Function evaluation has no Grid occupancy, directional destination, old-Footprint clear, collision delivery, or partial-unit alignment semantics. |
| Function evaluation | Deliberately does not reuse it; movement is a spatial Language Unit effect. | Reuses parsing/encoding only. Evaluation must be extended with a non-value, self-reproducing exception whose result destination is not the ordinary result position. |
| Runtime values and Sequence | Neither reused nor extended. Activation remains outside both domains. | Putting Activation in `Atom`/`Atoms` exposes it to operand stacks and future pervasive Sequence behavior, which then needs rejection rules at each value seam. |
| Source writes and generated Source | Reuses atomic Tick-plan writes and Source commit. Rebuilding the next Language Map recognizes generated Activation naturally, so it cannot move on its emission Tick. | Can reuse Atom-to-Source spelling, but still needs bespoke clear/write ordering and the same next-Snapshot scheduling guard. |
| Glyphs | The Language Map classifies the recognized Footprint directly; no Expression typing is needed. | Can reuse parser token Glyph conversion, but falsely makes presentation depend on an Activation-only Expression path. A true Function form would misclassify the character as a Function/root. |

The spatial model wins because every distinctive requirement concerns Source position, Footprint,
Grid occupancy, root alignment, or ordered effects. Those facts already meet at the Language
Map/Tick-planning interface. The value model saves only a spelling conversion while spreading
Activation exceptions across parsing, evaluation, result placement, operand validation, Sequence
extension, and Glyph derivation.

### Constraints for `spatial-tick-planning/03`

- Recognize all four spellings as two-Cell spatial Language Units carrying direction.
- Give only Activation Characters present in the Source Snapshot a row-major turn, keyed by their
  anchor Position. A generated character first receives a turn from the next Source Snapshot.
- Plan movement from the Language Map and Grid. Test only newly entered Cells; on success clear the
  complete old Footprint before writing the complete new Footprint.
- On collision or an out-of-Grid destination, replace the current Footprint with `**`. Deliver
  activation only to a completely aligned Expression root whose turn has not passed. Partial-unit
  contact diagnoses and never delivers.
- Keep direction visible solely in Source. Add no hidden cross-Tick state.
- Keep Activation out of `Atom`, Function signatures, Interpreter stacks/results, ordinary result
  writes, and Sequence extension. Share a direction/spelling type only if it does not reopen those
  capabilities.
- Derive Glyphs through the Language Map's recognized unit and reuse the common Tick-plan write,
  conflict, diagnostic, and Source-commit machinery.

The executable comparison model was intentionally throwaway and has been removed. Its assertions
covered the acceptance scenarios above. The retained declarative presentation makes the same
Tick-by-Tick Source Grids, diagnostics, and model comparison reviewable without preserving either
candidate implementation; this ticket records the decision and its production constraints.
