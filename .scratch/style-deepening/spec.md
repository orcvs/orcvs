# Style deepening

Refactors from the 2026-10-22 architecture review, each taking knowledge that callers repeat and putting it behind the module that already owns it. All start from `main`. None changes what the console paints or what a Theme document accepts, and none claims a speedup.

1. **The Cell colour decision** (`issues/01`). Paint (`console/src/paint.rs`) and contrast (`console/src/contrast.rs`) each derive the Cursor and Region state, choose selected or unselected visuals, prepare the Cursor, Region and base fills, and call `style::cell_background` with seven arguments whose relationship the caller must keep. That protocol moves into `style`.
2. **The Theme catalogue's routing** (`issues/02`). `style_catalogue!` in `console/src/theme.rs` already spells every property's variant, field and name, but the key enums and their read and write matches are still written by hand. The catalogue generates them.
3. **A width validated once** (`issues/03`). The Theme document decoder and `resolve` both check every width's bound; the decoder produces the typed width and `resolve` stops failing on one.
4. **A drawn line built once** (`issues/04`). ADR 0078's zero-width rule is restated at each place Paint and `style` build a line; one constructor states it.
5. **The document-only Theme items in one module** (`issues/05`). Twenty identical web `dead_code` attributes become the one `theme_document` already carries.

## Decisions

- 01 to 04 are independent; 05 waits for 02, since both move the key enums' routing.
- They land on `main` before `claude/grayscale-themes-7111p9`, which is rebased onto them: its Cursor glyph rule (`cell_foreground`) becomes a change inside `CellColours::colours`, and its role border properties become catalogue lines.
- The theme workbench (`.scratch/theming/workbench/`) is out of scope.
- ADR 0040 and ADR 0053 stand: edges stay in Paint, surface measurement stays in contrast, and no new ADR is needed.
- The simplest code wins: no new trait, no code-generation framework, no runtime registry, and no test-only parameter in shipped code.
- Portal precedence (Output over Input over Locked, ADR 0077) stays stated where it is: deriving it from `EdgeRank` touches the per-Cell walk for a small gain on an order that does not change.
