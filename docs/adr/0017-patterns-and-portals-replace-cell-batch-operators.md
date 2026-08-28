# Patterns and Portals replace Cell-batch Operators

Orcvs preserves the capabilities of Orca's Track, Generator, Query, Konkat, Push, and Variable operators without adopting them as separate Functions. Track becomes Pattern selection. Generator, Query, and Konkat become Source reads or terminal Source writes of one intact Pattern. Push becomes Pattern replacement followed by terminal Source write. Variable's hidden named table is omitted entirely: the Source is Orcvs's visible address space, and the Source Snapshot must contain all persistent language state.

This consolidation follows Orcvs values rather than Orca's single-Cell workarounds. It does not remove indexed selection, multi-Cell copying, replacement, or visible variable-like storage; it gives those capabilities one composable Pattern and Portal vocabulary.
