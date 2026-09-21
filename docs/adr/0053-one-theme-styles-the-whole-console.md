# One Theme styles the whole console

Status: accepted. `.scratch/theming/issues/01` carries it. It supersedes [ADR 0051](0051-a-theme-maps-facts-to-channels.md)'s affordance namespace, its storage rule, its Sequence assignment, and its concession of chrome to egui's `Visuals`. ADR 0051's framing stands: a Cell carries facts, the console draws them through channels, and `style.rs` stops choosing between them in control flow.

**Every part of the console's presentation is themeable, and one Theme styles all of it.** ADR 0051 scoped theming to the Source Grid and left the page, panels, widgets, selection, Cell grid lines and Sector Seams to egui's dark/light `Visuals`: two fixed palettes, compiled in, with nothing a viewer can choose or load. That was never the requirement. The Source Grid and the console around it are never themed separately, so loading a published base16 scheme restyles the menus as well as the Grid.

**Settings choose Themes by name, and a Theme is its own document.** Settings hold a dark Theme's name, a light Theme's name, and a mode: follow the operating system's appearance, or hold one of the two. They hold no Theme values. This is the arrangement Helix (`theme = "…"` plus one file per theme), Zed (`"theme": { "mode", "dark", "light" }`) and WezTerm (`color_scheme`) all use. Every Theme declares whether it is dark or light, and only a dark Theme can fill the dark slot. A base16 scheme that has no `variant` line is classified by the lightness of its `base00`. egui needs that answer anyway, for `Visuals::dark_mode`.

**There are no overrides.** A Theme is never adjusted in place. A viewer who wants a different look makes a custom Theme, which names exactly one built-in Theme it inherits from and lists only the values it changes, as Helix's `inherits` does. A custom Theme never inherits from another custom Theme, so there are no chains and a parent always exists. Built-in Themes are compiled in. Custom Themes and loaded schemes are stored as documents; on the web target a document is a storage entry. A built-in Theme and a custom one are the same type and appear in the same lists.

**A Theme decides how things look, never how much they move.** Glitch amount and Glitch frequency are settings, beside the operating system's reduced-motion preference, as cursor blink is in VS Code and Zed. A viewer who needs the Cursor Effect still should not have to make a custom Theme for every Theme they use. The Cursor Effect's colours, and the Fill tint, belong to the Theme.

**A Theme is sixteen base16 slots plus named keys, and every named key takes its default from a slot.** A bare published scheme sets only the slots and still styles the whole console. Built-in Themes set every key explicitly, which is how Okabe–Ito keeps today's appearance. Keys are dotted lowercase, element first and property second. Where the property is a channel it is named `background`, `border` or `foreground`, as in Zed's `panel.background` and VS Code's `editorCursor.foreground`. Opacity is part of every colour value, written `#RRGGBBAA`, and there are no separate opacity keys.

| Token | Slot | base16 role |
|---|---|---|
| Source background | `base00` | Default Background |
| Comment | `base03` | Comments |
| Ordinary, Char, Atom | `base05` | Default Foreground |
| Number | `base09` | Integers, Constants |
| Note | `base0B` | Strings |
| Function | `base0D` | Functions, Methods |
| Bang | `base0E` | Keywords, Storage |
| Sequence | `base0F` | Deprecated, Embedded Language Tags |

| Key | Default |
|---|---|
| `diagnostic.foreground` | `base08` |
| `diagnostic.background`, `diagnostic.border` | transparent |
| `output_portal.foreground` | `base0A` |
| `output_portal.background` | `base0A` at the Fill tint's opacity |
| `output_portal.border` | transparent |
| `fill_tint` | 16% |
| `grid.border` | `base03` at 28% |
| `sector.seam` | `base03` at 43% |
| `cursor.border` | `base05` |
| `cursor.area` | `base0C` |
| `cursor.background` | none |
| `region.background` | `base05` at 17% |
| `region.cursor.background` | none |
| `panel.background` | `base01` |
| `panel.border` | `base02` |
| `text` | `base05` |
| `text.muted` | `base04` |
| `input.background` | `base00` |
| `selection.background` | `base02` |
| `selection.border` | `base0C` |
| `selection.border.rest` | `base0C` at reduced opacity |
| `error` | `base08` |
| `warning` | `base09` |

**Diagnostic takes `base08` and Sequence moves to `base0F`.** ADR 0051 kept Diagnostic off the slots and gave `base08` to Sequence, by its "Markup Lists" role. Once every named key needs a default slot, Diagnostic needs one too. Among eight published schemes (Default Dark, Gruvbox Dark, Solarized Dark, Nord, One Dark, Tomorrow Night, Monokai, Dracula), `base08` is red in every one. `base0F` is brown, dark red, orange, magenta or blue, depending on the scheme. A Diagnostic has to read as an error, and a Token only has to be distinct from its neighbours, so the slot whose colour is reliable goes to the Diagnostic. The cost lands on Sequence and is accepted: in Nord, `base0F` is `#5E81AC`, a blue close to Function's `base0D`, and `.scratch/theming/issues/08`'s validator reports it rather than the template preventing it.

**The Theme picks a fact's channel through colour values, not through a mapping.** Diagnostic and Output Portal each take a key per channel, the way VS Code gives errors `editorError.foreground`, `.background` and `.border`. A Theme puts Diagnostic on the fill by setting `diagnostic.background` and making `diagnostic.foreground` transparent. A fact's channel paints over the Token's colour on the same channel, with opacity compositing, and a transparent channel leaves the Token's colour showing. That is how an invalid operand keeps its declared Token's tint. Tokens keep one glyph colour each, plus their Fill tint, because a base16 slot is a glyph colour by definition.

## Rejected alternatives

**Chrome as egui's dark/light `Visuals`.** This is what ADR 0051 did, and it left most of the console outside any Theme.

**A Source scheme and a chrome theme, chosen separately.** Every pairing would need its own contrast review, and chrome would need a Theme format of its own. One Theme derived from base16 gets coherent chrome from every published scheme for nothing.

**Per-role overrides keyed by scheme, which ADR 0051 stored.** An override is a second answer to what a colour is, and every reader has to resolve the two. A custom Theme that inherits gives the same "change one colour" edit with one answer.

**Diagnostic on `base0F`, keeping Sequence on `base08`.** A Diagnostic whose colour depends on the scheme does not reliably read as an error. In Nord it would be a blue beside Function's `base0D`.

**Separate opacity keys.** They give two answers to how transparent a colour is, and no theme format in the prior art uses them.

## Consequences

`SourcePaintSettings`, `CursorEffectSettings`' colours, `ConsolePalette`, and the `source_paint` storage key are all replaced rather than migrated, for the reason ADR 0051 gave: a resolved value has no room for "unset". The `cursor_effects` key keeps only Glitch amount and Glitch frequency, or moves them into the settings document.

`Theme → Source colours` and `Theme → Cursor effects` become a Theme picker for each of the two slots, a mode, and an editor that makes custom Themes. "Reset to theme defaults" has nothing left to reset.

`style()` builds egui's `Visuals` from the resolved Theme. It no longer reads a constant palette or Source defaults, so issue `05`'s borrow has nothing left to borrow.

A light appearance is a light built-in Theme, not a second chrome palette.
