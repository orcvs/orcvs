# Style deepening

Two refactors from the 2026-10-22 architecture review, each taking knowledge that callers repeat and putting it behind the module that already owns it. Both start from `main`. Neither changes what the console paints or what a Theme document accepts, and neither claims a speedup.

1. **The Cell colour decision** (`issues/01`). Paint (`console/src/paint.rs`) and contrast (`console/src/contrast.rs`) each derive the Cursor and Region state, choose selected or unselected visuals, prepare the Cursor, Region and base fills, and call `style::cell_background` with seven arguments whose relationship the caller must keep. That protocol moves into `style`.
2. **The Theme catalogue's routing** (`issues/02`). `style_catalogue!` in `console/src/theme.rs` already spells every property's variant, field and name, but the key enums and their read and write matches are still written by hand. The catalogue generates them.

## Decisions

- The two refactors are independent; either may land first.
- They land on `main` before `claude/grayscale-themes-7111p9`, which is rebased onto them: its Cursor glyph rule (`cell_foreground`) becomes a change inside `CellColours::colours`, and its role border properties become catalogue lines.
- The theme workbench (`.scratch/theming/workbench/`) is out of scope.
- ADR 0040 and ADR 0053 stand: edges stay in Paint, surface measurement stays in contrast, and no new ADR is needed.
- The simplest code wins: no new trait, no code-generation framework, no runtime registry, and no test-only parameter in shipped code.
