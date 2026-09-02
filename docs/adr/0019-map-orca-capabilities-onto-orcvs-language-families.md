# Map Orca capabilities onto Orcvs language families

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
| Vertical Jumper and horizontal Jymper | `J Y` | Generalize to four directional Jump Functions over complete Language Units | `&^ &v &< &>` | ADRs 0005 and 0014 |
| Number Range and Note Range | — | Add distinct monomorphic Sequence Functions | `:-` and `:#` | ADRs 0007 and 0023 |
| Reverse | — | Add as a first-class Sequence Function | `:<` | ADR 0007 |
| Concatenate | — | Add as a first-class Sequence Function | `:&` | ADR 0007 |
| Track | `T` | Replace with Sequence Select | `:?` | ADRs 0007 and 0017 |
| Push | `P` | Replace with Sequence Replace followed by Source Write | `:=` then `@>` | ADRs 0007 and 0017 |
| Read and Query | `O Q` | Replace with Source Read returning an Atom or Sequence | `@<` reserved; operands deferred | ADRs 0005 and 0017 |
| Write | `X` | Replace with Source Write through a Portal | `@>` reserved; operands deferred | ADRs 0004, 0005, 0009, and 0017 |
| Generator | `G` | Compose Source Read and Source Write | reserved `@<` then `@>`; operands deferred | ADR 0017 |
| Konkat | `K` | Replace hidden-variable lookup with Source Read from visible Source | `@<` reserved; operands deferred | ADRs 0003 and 0017 |
| Variable | `V` | Omit the hidden named table; persistent language state remains in Source | — | ADRs 0003 and 0017 |
| Comment | `#` | Retain | `#` | Existing language contract |
| Raw MIDI note | `:` | Retain without an implicit lifetime | `!>` | ADR 0016 |
| Timed MIDI note | optional Orca `:` lifetime | Make a distinct fixed-arity Function | `!~` | ADR 0016 |
| Monophonic MIDI note | `%` | Retain with one owned voice per adapter and channel | `!%` | ADRs 0008 and 0016 |
| MIDI Control Change | `!` | Retain with direct hexadecimal MIDI bytes | `!c` | ADRs 0008 and 0016 |
| MIDI Pitch Bend | `?` | Retain as direct LSB and MSB bytes | `!b` | ADRs 0008 and 0016 |
| UDP | `;` | Defer until Orcvs has a text or message value | `!u` reserved | ADRs 0008 and 0016 |
| OSC | `=` | Defer until Orcvs has a text or message value | `!o` reserved | ADRs 0008 and 0016 |
| Orca self command | `$` | Retain as an Orcvs Application Command with no shell or process execution | `!$`; value encoding deferred | ADR 0008 |
| Identity Test | prior Orcvs `id` | Retire; use Equality or ordinary value flow | — | ADR 0015 |

The canonical families are numeric `.`, Tick and feedback `~`, activation `*`, address `&`, Source `@`, Sequence `:`, and terminal output `!` (ADR 0008). General arithmetic takes and returns Numbers; `.v` and `.^` explicitly convert between Number and Note with fixed result types (ADRs 0010, 0011, and 0021). Sequence broadcasting and structural behavior follow ADR 0007; editing remains a one-character Cell Grid with semantic behavior derived through the Language Map (ADR 0018).

Two design boundaries remain intentionally deferred rather than unresolved in this audit: the concrete Source address form beyond directional Jump (ADR 0005), and the text or message value encoding required by UDP, OSC, and Application Command. Contextual two-Cell Note interpretation and the distinct Number and Note Range names are settled by ADRs 0021 and 0023. The Orca-to-Orcvs capability map is a presentation and implementation-tracking view of this decision, not a separate source of language truth.
