# The Overview is called the Minimap

Status: accepted. Renames the term ADRs 0080, 0081 and 0096 introduced and placed. Decided by Toby on
2026-10-31.

The reduced picture of the whole Grid in the Source's corner was named the Overview, and CONTEXT.md
listed "minimap" as a term to avoid, because "map" might be read as the Language Map. People call
it the minimap anyway, and the name is the usual one for this element in editors.

## Decision

**The term is Minimap.** CONTEXT.md, the ADRs, the console's code, the View menu item, the
accessible label "Source minimap", the Theme keys `minimap.source_view` and
`minimap.source_view.width`, the settings table `[minimap]` and the benchmark
`paint_minimap/picture/256x256` all use it. "Overview" moves to the avoided terms. "Map" stays
avoided on its own, so the Language Map keeps it.

**The old names are not read.** The language is pre-release, with no compatibility contract. A
Theme that sets `overview.source_view` is refused, as a Theme with any unknown key is. A settings
file with `[overview]` gets the unknown-key notice and the default applies.

## Consequences

- The benchmark series starts again under its new name. The floor carries over unchanged.
- ADRs 0080, 0081 and 0096 keep their numbers. Their file names and text now say Minimap.
