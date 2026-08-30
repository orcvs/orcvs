# Sequences and Portals replace Cell-batch Operators

Orcvs preserves the capabilities of Orca's Track, Generator, Query, Konkat, Push, and Variable operators without adopting them as separate Functions. Track becomes Sequence selection. Query becomes a Source Read that returns one Sequence. Generator becomes a Source Read followed by a Source Write of one intact Sequence. Konkat becomes a Source Read from visible Source instead of a hidden variable table. Push becomes Sequence replacement followed by Source Write. Variable's hidden named table is omitted entirely: the Source is Orcvs's visible address space, and the Source Snapshot must contain all persistent language state.

This decision maps capabilities, not concrete operands. The Source address form remains deferred by ADR 0005, so `@<` and `@>` are reserved spellings rather than parseable Functions until that contract is accepted.

This consolidation follows Orcvs values rather than Orca's single-Cell workarounds. It does not remove indexed selection, multi-Cell copying, replacement, or visible variable-like storage; it gives those capabilities one composable Sequence and Portal vocabulary.
