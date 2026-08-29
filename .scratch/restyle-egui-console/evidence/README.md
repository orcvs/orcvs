# Console restyle evidence

- `console-wide.png`: WASM console rendered at 1200×700. The square Source Grid is centred with horizontal letterboxing.
- `console-tall.png`: WASM console rendered at 700×1200. The square Source Grid is centred with vertical letterboxing.

The exact palette is recorded in the console theme documentation and declared by the console style boundary.

## Remaining prototype differences

The prototype's explanatory panels, scenario tabs, diagnostic prose, Tick controls, rounded chrome, and recently-changed outlines are intentionally absent: they are prototype scaffolding rather than the Orcvs console. Runtime diagnostics remain in the existing MIDI/status presentation, using the palette's soft red. A collision continues to become a Source Bang and is painted soft red; there is no separate collision tile state to add without changing the console's presentation model.
