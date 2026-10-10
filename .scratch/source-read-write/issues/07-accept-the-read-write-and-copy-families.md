# 07 — Accept the Read, Write and Copy families

Status: resolved
Type: grilling

**What to build:**

A human review of [ADR 0070](../../../docs/adr/0070-read-write-and-copy-are-separate-families.md), which proposes three families by action: Read `&`, Write `@` and Copy `=`, with Jump renamed Copy, Track moved to `&t`, Push at `@t`, and absolute forms spelled with `$`. The decisions it records were made by the user; the review settles the point it marks **To confirm** and accepts it. Every ticket that changes a spelling waits on this one.

## Acceptance criteria

- [x] Where each direction counts from is settled: distances count Portals from the `n` operand, which is `00`.
- [x] How Sources written with the old spellings are handled is confirmed or changed. The proposed default: they are read under the new grammar, and the console's stored autosave is discarded once with a report (ticket 12).
- [x] ADR 0070's Status line reads accepted, and its text records the confirmed answer in place of the proposed default.
- [x] ADRs 0008, 0019, 0049 and 0067 each carry a Status-line amendment pointing at ADR 0070 and a dated amendment section stating the change. ADR 0019's table and `CONTEXT.md` name every Read, Write and Copy from acceptance, because they describe the language the ADRs define; the glossary's Output Portal entry still waits on ticket 04.
- [x] If a confirmed answer changes a ticket's acceptance criteria, that ticket is updated in the same change.

## Resolution

The user confirmed the proposed default: old spellings are read under the new grammar and the console's stored autosave is discarded once (ticket 12). ADR 0070 is accepted. Because the glossary and ADR 0019's table now change at acceptance, tickets 02, 05, 06, 08, 09, 10 and 11 and the spec's point 13 keep only the Function reference and shipped Source Files.
