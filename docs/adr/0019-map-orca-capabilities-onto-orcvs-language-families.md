# Map Orca capabilities onto Orcvs language families

Status: the Sequence mappings are superseded by [ADR 0063](0063-a-list-is-cells-in-a-claim-not-a-value.md). The capability audit stands. Its rows for Jump, Track, Push, Read and Query, Write, Generator and Konkat are amended by [ADR 0070](0070-read-write-and-copy-are-separate-families.md) (2026-10-10 amendment below).

Orcvs preserves Orca's performative capabilities without preserving its one-letter Operator encoding or its uppercase/lowercase scheduling convention. Capabilities are expressed through two-character, behavior-first Function families over the character Source and derived Language Map; the detailed contracts remain in the focused ADRs linked below, while this ADR is the authoritative audit index. Tick-wide effect ordering is defined by ADR 0020.

| Capability | Orca | Orcvs decision | Canonical Orcvs form | Detail |
| --- | --- | --- | --- | --- |
| Addition | `A` | Retain | `.+` | ADR 0011 |
| Ordered subtraction | — | Add | `.-` | ADR 0011 |
| Absolute difference | `B` | Retain separately from subtraction | `.\|` | ADR 0011 |
| Multiplication | `M` | Retain | `.x` | ADR 0011 |
| Division | — | Add | `./` | ADR 0011 |
| Modulo | — | Add | `.%` | ADR 0011 |
| Minimum | `L` | Retain | `.<` | ADR 0011 |
| Maximum | — | Add | `.>` | ADR 0011 |
| Equality | `F` | Retain as Bang-producing comparison | `.=` | ADRs 0006 and 0011 |
| Note to Number | — | Add explicit, total conversion | `.v` | ADR 0021 |
| Number to Note | — | Add explicit, checked conversion | `.^` | ADR 0021 |
| Clock | `C` | Retain with explicit Tick input | `~.` | ADR 0012 |
| Delay | `D` | Retain with explicit Tick input | `~*` | ADR 0012 |
| Increment | `I` | Retain with visible feedback | `~+` | ADR 0012 |
| Random | `R` | Retain with explicit deterministic seed | `~?` | ADR 0013 |
| Euclidean rhythm | `U` | Retain with explicit Tick input | `~%` | ADR 0012 |
| Interpolation | `Z` | Retain with visible feedback | `~>` | ADR 0012 |
| North, south, west, and east movement | `N S W E` | Use Directional Bang Functions to emit Source-resident Self-Banging Functions | `*^ *v *< *>` emit `^^ vv << >>` | ADRs 0006 and 0014 |
| Bang | `*` and lowercase case | Retain as explicit two-Cell Atom; remove Function case | `**` | ADR 0006 |
| Halt | `H` | Retain with Orca Source-order behavior | `*!` | ADRs 0006 and 0014 |
| Vertical Jumper and horizontal Jymper | `J Y` | Generalize to four directional Copy Functions over complete Language Units | `=^ =v =< =>` | ADRs 0005, 0014 and 0070 |
| Number Range and Note Range | — | Add distinct monomorphic Sequence Functions | `:-` and `:#` | ADRs 0007 and 0023 |
| Reverse | — | Add as a first-class Sequence Function | `:<` | ADR 0007 |
| Concatenate | — | Add as a first-class Sequence Function | `:&` | ADR 0007 |
| Track | `T` | Retain as a Read of the pair an index selects along a row | `&t index count` | ADRs 0067 and 0070 |
| Push | `P` | Retain as a Write into the pair an index selects in the lane below it | `@t index count value` | ADRs 0067 and 0070 |
| Read and Query | `O Q` | Replace with a directional Read `n` Portals from its operand, or an absolute Read at a Position; one Atom per Read, so a multi-Atom Query is several Reads | `&^ &v &< &> n`, `&$ column row` | ADRs 0049, 0063, 0067 and 0070 |
| Write | `X` | Replace with a directional Write `n` Portals from its operand, or an absolute Write at a Position | `@^ @v @< @> n value`, `@$ column row value` | ADRs 0049 and 0070 |
| Generator | `G` | Compose a Read nested in a Write; `=$` copies one pair between two Positions | Read in a Write's `value`; `=$ src-column src-row dst-column dst-row` | ADRs 0017 and 0070 |
| Konkat | `K` | Replace hidden-variable lookup with a Read of visible Source, one Atom per Read | `&^ &v &< &> n`, `&$ column row` | ADRs 0003, 0063 and 0070 |
| Variable | `V` | Omit the hidden named table; persistent language state remains in Source | — | ADRs 0003 and 0017 |
| Comment | `#` | Retain | `\|\|` | Existing language contract and ADR 0035 |
| Raw MIDI note | `:` | Retain without an implicit lifetime | `!>` | ADR 0016 |
| Timed MIDI note | optional Orca `:` lifetime | Make a distinct fixed-arity Function | `!~` | ADR 0016 |
| Monophonic MIDI note | `%` | Retain with one owned voice per MIDI channel | `!%` | ADRs 0008 and 0016 |
| MIDI Control Change | `!` | Retain with direct hexadecimal MIDI bytes | `!c` | ADRs 0008 and 0016 |
| MIDI Pitch Bend | `?` | Retain as direct LSB and MSB bytes | `!b` | ADRs 0008 and 0016 |
| UDP | `;` | Defer until Orcvs has a text or message value | `!u` reserved | ADRs 0008 and 0016 |
| OSC | `=` | Defer until Orcvs has a text or message value | `!o` reserved | ADRs 0008 and 0016 |
| Orca self command | `$` | Retain as an Orcvs Application Command with no shell or process execution | `!$`; value encoding deferred | ADR 0008 |
| Identity Test | prior Orcvs `id` | Retire; use Equality or ordinary value flow | — | ADR 0015 |

The canonical families are numeric `.`, Tick and feedback `~`, activation `*`, address `&`, Source `@`, Sequence `:`, and terminal output `!` (ADR 0008). General arithmetic takes and returns Numbers; `.v` and `.^` explicitly convert between Number and Note with fixed result types (ADRs 0010, 0011, and 0021). Sequence broadcasting and structural behavior follow ADR 0007; editing remains a one-character Cell Grid with semantic behavior derived through the Language Map (ADR 0018).

This audit deferred two design boundaries intentionally rather than leaving them unresolved: the concrete Source address form beyond directional Jump (ADR 0005, which [ADR 0049](0049-a-position-is-two-numbers.md) has since settled: a Position is two Numbers), and the text or message value encoding required by UDP, OSC, and Application Command. Contextual two-Cell Note interpretation and the distinct Number and Note Range names are settled by ADRs 0021 and 0023. The Orca-to-Orcvs capability map is a presentation and implementation-tracking view of this decision, not a separate source of language truth.

## Amendment, 2026-10-10: Read, Write and Copy families

[ADR 0070](0070-read-write-and-copy-are-separate-families.md) splits the address family `&` and the Source family `@` into Read `&`, Write `@` and Copy `=`, and the table above states the result. Seven rows changed. Jump `J Y` was `&^ &v &< &>` and is now the Copy Functions `=^ =v =< =>`, with behaviour unchanged. Track `T` still named the retired Sequence Select `:?`; it maps to `&t index count`, as ADR 0067 built it at `@t`. Push `P` was Sequence Replace followed by Source Write `:=` then `@>`; it is now `@t index count value`. Read and Query `O Q` and Konkat `K` were the relative two-axis Read `@< column row`; they are now the directional Reads and the absolute Read `&$`. Write `X` was `@>` with operands deferred; it is now the directional Writes and the absolute Write `@$`. Generator `G` stays a Read nested in a Write, and `=$` copies between two Positions in one Function. In the paragraph after the table, read the address family `&` as the Read family, the Source family `@` as the Write family, and add the Copy family `=`.
