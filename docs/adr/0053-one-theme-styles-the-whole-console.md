# One Theme styles the whole console

Status: accepted. `.scratch/theming/issues/01` carries it. The web target's Theme import is disabled for v1 (`.scratch/menu-structure/issues/04`): the web reads no settings file, so nothing there could select an imported Theme, and the web console has the built-ins alone. "Web Themes are imported as files" below remains the design for when web import returns. It supersedes [ADR 0051](0051-a-theme-maps-facts-to-channels.md)'s affordance namespace, its storage rule, its palette-slot representation, and its concession of chrome to egui's `Visuals`. ADR 0051's framing stands: a Cell carries facts, the console draws them through channels, and `style.rs` stops choosing between them in control flow.

**Every part of the console's presentation is themeable, and one Theme styles all of it.** ADR 0051 scoped theming to the Source Grid and left the page, panels, widgets, selection, Cell grid lines and Sector Seams to egui's dark/light `Visuals`: two fixed palettes, compiled in, with nothing a viewer can choose or load. That was never the requirement. The Source Grid and the console around it are never themed separately, so loading an Orcvs Theme restyles the menus as well as the Grid.

**Settings choose Themes by name, and a Theme is its own document.** Settings hold a dark Theme's name, a light Theme's name, and a mode: follow the operating system's appearance, or hold one of the two. They hold no Theme values. This is the arrangement Helix (`theme = "…"` plus one file per theme), Zed (`"theme": { "mode", "dark", "light" }`) and WezTerm (`color_scheme`) all use. Every Theme declares whether it is dark or light, and only a dark Theme can fill the dark slot. Custom Themes inherit that appearance from their built-in parent.

**There are no overrides.** Settings never adjust a Theme's values. A viewer who wants a different look authors or edits a custom Theme file outside Orcvs. It names exactly one built-in Theme it inherits from and lists only the values it changes, as Helix's `inherits` does. A custom Theme never inherits from another custom Theme, so there are no chains and a parent always exists. Built-in Themes are compiled in. Custom Themes are stored as documents; on the web target a document is a storage entry. A built-in Theme and a custom one are the same type and appear in the same lists.

**Inheritance preserves explicit parent values.** A custom Theme inherits its parent's resolved named properties and replaces only properties explicitly supplied by the document. Omitted properties remain unchanged; no palette-slot recalculation exists.

**Custom appearance follows the parent.** A custom Theme inherits its built-in parent's dark/light appearance. An omitted appearance resolves from the parent; an explicit conflicting declaration rejects the document with an error identifying the mismatch. A light custom Theme must start from a light built-in, and a dark custom Theme from a dark built-in.

**Making a Theme is a file change.** Orcvs loads and selects Theme documents; it has no in-app Theme editor. Colour pickers, live authoring previews and automatic creation from the currently displayed Theme are not part of this effort. This corrects the editor scope previously stated here, following the user's explicit clarification on 2026-09-21.

**Native Theme files are authoritative and read at startup.** Custom Themes and loaded schemes live in `~/.orcvs/themes/`, the canonical native Theme directory. At each startup Orcvs scans that directory and reads its Theme files, so edits made since the previous run take effect without importing them again. Settings select Themes by identity, not by storing cached copies of their values. Orcvs does not also search XDG or other Theme directories: one location avoids search-precedence rules. Native file edits during a running session take effect only on the next launch for this release; there is no file watcher or reload action. On the web target, an imported document remains a storage entry.

**A file Theme's identity is its filename stem on both targets.** The filename without its extension is what settings reference; a `name` inside the document is only a display label. Changing that label does not change the selection, while renaming the file changes its identity. Built-in Theme identities are reserved: a file using one is refused and reported rather than replacing the built-in Theme. These rules apply to Orcvs Theme files.

**Duplicate native identities are conflicts.** If multiple native files have the same filename stem, all files with that identity are refused and the conflict is reported with the conflicting filenames. The stem keeps its case while the extension is matched case-insensitively, so `Dark.toml` and `dark.toml` are two identities, and `dark.toml` and `dark.TOML` are one identity in conflict — which only a case-sensitive file system can hold. Files in one web drop that share a stem, including the same name dropped twice, are refused and reported the same way. Directory enumeration order never chooses a winner. If the selected identity is conflicted, use the default built-in Theme of the same appearance while retaining the saved selection; resolving the conflict restores the intended Theme on the next launch. This differs from an explicit web reimport, which updates an existing imported document.

**Web Themes are imported as files.** The web target uses a file import rather than pasted text. The filename stem identifies the imported Theme; its document name remains a display label and built-in identities remain reserved. Reimporting the same identity updates its document after successful validation. Imported documents are retained in browser storage when persistence is enabled; otherwise they are session-only. A stored `.toml` document that no longer loads is kept and refuses its identity with the reason.

**A missing or malformed selected Theme does not prevent startup.** Orcvs uses the default built-in Theme for the selected appearance and shows an error identifying the unavailable Theme and the file problem. This fallback is only the effective appearance: it never replaces the saved Theme selection, including on autosave. Restoring or fixing the file therefore restores the selected Theme on the next launch without selecting it again. Malformed documents are refused whole; contrast warnings alone do not make a document malformed.

**Theme settings are validated strictly.** Unknown appearance keys and out-of-range widths reject the entire Theme document. The error identifies the offending setting and explains the valid key or range; values are neither silently ignored nor clamped. An unknown property's error names the unknown key and its line, and starts listing the valid property names, colours first; the 1024-byte message cap cuts that list, so it may omit the width and optional-fill names. It no longer suggests the property a near miss in case or separator meant ("did you mean"). That hint needed a hand-written key parser in front of the derived `style` table; the key and its position locate the mistake, and `schema.md`'s catalogue has the complete spellings. A failed web reimport preserves the previous valid document. Contrast warnings alone do not reject a document.

**A Theme decides how things look, never how much they move.** Glitch amount and Glitch frequency are settings, beside the operating system's reduced-motion preference, as cursor blink is in VS Code and Zed. A viewer who needs the Cursor Effect still should not have to make a custom Theme for every Theme they use. The Cursor Effect's colours and role background colours belong to the Theme.

Amount zero gives a clear, stationary Cursor frame and no cursor-effect repaint
deadline, even with positive frequency. Frequency zero freezes the current
effect, potentially retaining decorative fragmentation, and likewise schedules
no cursor-effect repaints. Reduced motion produces a clear, stationary frame
without replacing the stored motion preferences. Playback, Run Clock and input
repaints remain independent. Stopping effect repaints at amount zero is an
intentional correction to the existing frequency-only scheduling rule.

**The planned controls are colours, opacity, Grid and Cell colours, borders and border widths.** Theme changes preserve Grid structure and input behaviour; painting and hit-testing must remain consistent. Grid background and Cell fills expose colour and opacity. Cell grid lines and Sector Seams each expose independent colour, opacity and width. Cursor, Region and Diagnostic borders expose colour, opacity and width. Widths are bounded; transparent colours can hide lines. Grid line and border widths are measured in display points and retain the same visible thickness as Grid zoom changes; they do not scale with Cell size. Existing square Cells, zoom and spacing remain unchanged. Exact key spellings and defaults are specified in `schema.md`; width bounds and existing stroke defaults are recorded below. Motion settings remain separate. Use the complete schema catalogue for implementation.

**Stroke widths have bounded display-point values.** Grid/Cell border and Sector Seam widths accept finite values from 0 to 1 display point inclusive; chrome border widths accept 0 to 2 points inclusive. Width zero hides the stroke. Preserve normal-zoom defaults: Cell grid lines 0.5 points, Sector Seams 0.75 points, stationary Cursor/Region effect outlines 1 point, and existing visible chrome borders 1 point (absent borders remain absent). Fixed display-point widths intentionally replace the previous zoom-scaled stroke behaviour.

**Animated strokes vary around the nominal width.** Cursor/Region effect width is nominal: retain the existing animated fragment variation of 0.45–1.25 times that width, without Grid zoom scaling. The 1-point maximum constrains nominal width, so an animated fragment can reach 1.25 points. Width zero hides every affected stroke; it does not implicitly disable separately coloured fills or change motion preferences.

**Opacity is within the console.** Transparency reveals the underlying console surface; the application window remains opaque. Desktop/window transparency is outside this effort. Painting and contrast validation must use the same composited backgrounds.

**Background layers have distinct colours.** The window backdrop and Grid background have separate Theme colours. The window backdrop must be opaque; panel, Grid and Cell layers above it may use alpha. A transparent Grid reveals the underlying console surface. Built-ins explicitly define both backgrounds; custom documents inherit them independently. Contrast validation uses the actual composite down to the opaque backdrop.

**Cells share a uniform base fill.** `cell.background` is a single Theme-wide base fill for all Cells, transparent by default. It composites over the Grid background and beneath role backgrounds, Diagnostic/Output Portal channels and Cursor/Region fills. It does not count as a fact fill when evaluating Region fallback, so setting it cannot suppress Region highlighting. Existing precedence among those highlighting channels remains unchanged. Individual Cells do not store styling.

**Role backgrounds are named colours.** Role backgrounds are explicit colour properties, such as `source.function.background`, `source.number.background`, `diagnostic.background` and `output_portal.background`. They composite over the uniform `cell.background` under the confirmed fixed precedence. Changing a role foreground does not recalculate its background. There is no `fill_tint` property or shared tint-strength setting in the Orcvs Theme format. Extract and verify the built-in role background colours from current rendering so its appearance is preserved; illustrative colours in the interview are not accepted defaults.

**Optional fills distinguish absence from transparency.** Optional Cursor fills distinguish omission, none and explicit transparent colour. Omission inherits the parent value. None removes the optional fill and uses the existing fallback: for the Cursor inside a Region, it falls back to the ordinary Cursor fill; for the ordinary Cursor, it supplies no Cursor fill. An explicit transparent colour remains a supplied value and does not trigger that fallback. These optional states apply only to `cursor.background` and `region.cursor.background`, not to every colour key.

**Leave room for future appearance controls without building them.** Font choice is a nice-to-have, not planned work. Font sizes, spacing, corner radii, shadows and gradients are also outside this effort. This supersedes the earlier typography, custom-font-file and broader visual-geometry scope. Keep the existing fonts and layout. Avoid new assumptions that would prevent future extensions, without adding speculative fields, loaders or abstractions. No future font format, source or delivery mechanism is committed by this decision.

**One versioned Orcvs Theme data model, in TOML.** Documents declare `format = "orcvs-theme"` and `version = 1`, with `name`, `inherits` and a `style` table of named properties. A Theme document is a `.toml` file and nothing else; the extension is matched case-insensitively, and every other extension, `.json`, `.yaml` and `.yml` included, is not a Theme file. `toml`'s own Serde deserializer decodes it directly into a derived, strict document type: the root and `style` deny unknown fields, `style` holds one optional field per property generated from the same declaration as the property keys' spellings, and each value deserializes into a newtype that validates it — the format marker, the version, labels, the appearance, colours, widths and the optional Cursor fills. TOML's parser refuses repeated keys, and a refusal carries its line, column and key path. Orcvs does not define or parse another configuration language, and no configuration framework sits in between. Each Theme is one source document; layering and environment overrides are not part of the Theme format. There is no `palette` section or required sixteen-slot representation. Built-ins provide complete named-property definitions; custom documents inherit exactly one built-in and specify changes. Missing or unsupported format/version markers reject the document with an explanatory error. Colours use hex RGBA, widths use the confirmed display-point units, and optional Cursor fills retain their explicit-clear semantics. The complete contract is [schema.md](../../.scratch/theming/schema.md), including exact keys, dark defaults and decoding rules.

**Base16 import is deferred.** Base16 can inspire built-in colours but is not a native file format or required internal model. A future importer would convert an external scheme into an Orcvs Theme document. No Base16 parser, slot table, template or additional published-scheme built-ins are required in this effort. This supersedes the earlier sixteen-slot and import decisions on 2026-09-22.

**The Theme picks a fact's channel through colour values, not through a mapping.** Diagnostic and Output Portal each take a key per channel, the way VS Code gives errors `editorError.foreground`, `.background` and `.border`. A Theme puts Diagnostic on the fill by setting `diagnostic.background` and making `diagnostic.foreground` transparent. A fact's channel paints over the Token's colour on the same channel, with opacity compositing, and a transparent channel leaves the Token's colour showing. That is how an invalid operand keeps its declared Token's background. Tokens have independently named foreground and background colours.

**Painting precedence is fixed, not a Theme property.** Theme files choose colours and channels within the existing Source-paint rules; they cannot reorder facts or change which fact takes precedence. A bound Function retains its paint inside an Output Portal. Bang retains its glyph colour while taking the portal tint. Otherwise Output Portal paint takes precedence over the underlying claim's paint, including Diagnostic. The Cursor's own fill retains precedence over role background. In a multi-Cell Region, the Cursor uses the Region's cursor fill, falling back to the Cursor fill when unset; other Cells use Region fill only when they have no cell fill. These rules preserve the existing appearance while channel colours gain alpha compositing as described above. The implementation must document and test the overlap cases, including transparent, partial-alpha and opaque channels; no Theme document carries precedence settings.

**Contrast validation measures painted text states.** Validate the resolved Theme's effective foreground against the actual composited background, including role backgrounds, selection, Cursor and Region fills, and console text on panel and input backgrounds. Follow the same fixed composition rules as painting and identify each result by role and state. A transparent fact foreground leaves the underlying Token visible; validate the resulting glyph, not the disabled channel. Text/background contrast does not assess whether two Token colours are distinguishable, nor does it constitute a complete accessibility assessment.

**Contrast acceptance preserves Okabe–Ito's Sequence colour.** Sequence remains `#0072B2` on `#000000`, approximately 4.05:1 against the 4.5:1 floor. Preserving the decided appearance takes precedence over making this one measurement pass. The validator still reports it as a failure, annotated as an accepted exception. Tests over shipped Themes reject additional unrecorded failures; this acceptance neither lowers the floor nor exempts other roles, Themes or newly measured failing states. Additional failures discovered by composited-state validation need explicit review.

### Known dark failures awaiting acceptance

The following reachable invalid-operand states are known below-floor results,
not accepted exceptions. Opaque Diagnostic foreground overlays the declared role
background outside an Output Portal and without a Cursor/Region fill replacing it.

| State | Foreground | Background | Calculated ratio | Floor | Status |
|---|---|---|---:|---:|---|
| Invalid Number operand | `#D55E00` | `#0E1D25` | 4.446944:1 | 4.5:1 | Explicit acceptance pending |
| Invalid Note operand | `#D55E00` | `#26240B` | 4.053689:1 | 4.5:1 | Explicit acceptance pending |

Recorded 2026-09-22 from the specified opaque colours using the standard sRGB
relative-luminance ratio. Confirm both through shipped rendering/composition in
the implementation. Exact dark appearance preservation cannot satisfy the shipped
Theme acceptance test until these failures are explicitly accepted; retain the
failures and do not silently whitelist them, lower the floor or retune colours.
The existing Sequence exception remains the only accepted exception. The table
is not an exhaustive claim: newly discovered failures still require review.

## Rejected alternatives

**Chrome as egui's dark/light `Visuals`.** This is what ADR 0051 did, and it left most of the console outside any Theme.

**A Source scheme and a chrome theme, chosen separately.** Every pairing would need its own contrast review, and chrome would need a Theme format of its own. One resolved Theme supplies both Source and chrome properties.

**Per-role overrides keyed by scheme, which ADR 0051 stored.** An override is a second answer to what a colour is, and every reader has to resolve the two. A custom Theme that inherits gives the same "change one colour" edit with one answer.

**Separate opacity keys.** They give two answers to how transparent a colour is, and no theme format in the prior art uses them.

**JSON and YAML Theme documents, beside TOML.** Accepted until 2026-09-24 and dropped then. Three representations of one model gave a viewer no Theme they could not write in TOML, and cost a third-party decoder per format that had to be made to agree: `serde-saphyr` parsed a quoted YAML scalar as a number on its typed paths and brought nine crates of its own, among them `unsafe` code and a proc macro, and `serde_json` kept the last of a repeated key and accepted a root array. Agreement took hand-written visitors for every scalar and for `style`, a root guard, and a test matrix run once per format, and four extensions let two files of one stem collide across formats. TOML is typed, always has a table root and refuses repeated keys, so a derived document type is strict without them. `serde_json` and `serde-saphyr` left the console with them.

## Consequences

Theme switching becomes available only when the Source Grid and chrome both
follow the selected Theme. Foundation work may land in separate pull requests,
but no usable intermediate state may apply a new Theme to only one part of the
console. This includes selection restored at startup, operating-system appearance
changes and loaded documents as well as picker interactions. Issue `06` supplies
the foundation; `03` connects chrome and prepares the Theme pickers while
retaining `02`'s same resolved presentation in both egui appearance slots. Issue `04`
prepares the existing light-palette proposal as a complete Theme, with contrast
results and visual captures for user review. Switching, including the pickers
and mode control, is exposed only after that review accepts the light Theme and
the whole console follows the selection. `04` owns distinct dark/light
registration and switching acceptance. The proposal is not approved merely
because this delivery sequence is agreed.

`SourcePaintSettings`, `CursorEffectSettings`' colours, `ConsolePalette`, and the `source_paint` storage key are all replaced rather than migrated, for the reason ADR 0051 gave: a resolved value has no room for "unset". The `cursor_effects` key keeps only Glitch amount and Glitch frequency, or moves them into the settings document.

`Theme → Source colours` and `Theme → Cursor effects` become a Theme picker for each of the two slots, a mode, and loading of externally authored Theme documents. "Reset to theme defaults" has nothing left to reset.

`style()` builds egui's `Visuals` from the resolved Theme. It no longer reads a constant palette or Source defaults, so issue `05`'s borrow has nothing left to borrow.

A light appearance is a light built-in Theme, not a second chrome palette.
