# Bang Function — reviewed proposal

**State:** settled by ticket 07 (D1–D17); ready for implementation. `spec.md` is the implementation contract and `docs/adr/0072-a-standalone-bang-is-a-function.md` the language decision. Existing ADRs remain authoritative for shipped behaviour until implementation accepts ADR 0072.

## Design constraint

The Bang Function is a new Function that follows the rules every Function follows. Every other outcome emerges from existing rules applied to the new root; no Bang-specific ordering, clearing or placement rule is added. Three read-only audits checked the proposal against the shipped language; ticket 07 records their findings.

## What is new

- A Function whose Turn activates its aligned roots and vacates its own Span. Its content is today's start-of-Tick fire, moved into the schedule.
- The parser's existing root-only `**` arm names that Function.
- Removals: the provenance record, the start-of-Tick clear, contact activation, and the missed-Bang diagnostic's restriction to dynamic writes.

## Orca reference and departures

Orca's `*` erases itself at its own turn; neighbours detect the glyph at theirs. Orcvs delivers activation when the Bang Function executes.

| Interaction | Orca | Orcvs | Reason |
|---|---|---|---|
| Typed Bang | Earlier north/west neighbours run; later south/east miss the erased glyph | Every aligned root activates (already ships) | Activation is delivered by the Bang's Turn, not detected by neighbours |
| Blocked mover | South neighbour runs in the collision frame, north/west next frame | `**` written; every aligned root activates next Tick | Contact activation removed; the `**` is an ordinary Bang Function |
| Mover meeting a typed Bang | West/north enter after the erase; east/south meet the glyph | Same: Source order decides | ADR 0060 rule 3b, no Bang-specific edge |
| Writer removed, halted, invalid or redirected after a pulse | Earlier neighbour can activate again | The surviving `**` fires once | No provenance; same rule as a removed writer |
| Halt above typed Bang | `H` runs first and locks the `*` | Halt inert, `**` fires, missed Bang at the Halt | Activation only by delivery; `lock_covers` |
| Locked Bang | Stands; neighbours detect it every frame | Stands; delivers no activation while locked | Activation requires its Turn |
| Copy under a locked Bang | `J` copies the standing `*` every frame (not probed) | `=v` carries `**` every Tick | Readers see the locked Cells |
| Copy with partial input | No counterpart | Writes nothing; old `**` fires once | ADR 0069 |

`evidence/README.md` describes the runnable Orca probes and the recovered reference's provenance limitations. Probes replace only `z`.

## Why remove provenance

The record lets identical Sources at the same Tick plan differently. Its anchor can disagree with a reparsed `***`, and a Bang in an operand loses its record after a quiet Tick. Removing it restores ADR 0003; the accepted price is surviving-Bang replay.

## Completion

Tickets 01–04 and 06 are one completion unit; 01 and 03 wait for placement-semantics 03. ADR 0072 is accepted and CONTEXT.md updated with the implemented contract.
