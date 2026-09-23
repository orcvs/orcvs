# egui theming and an alignment path for the console

## Recommendation

Align with egui's native model now: support one `ThemePreference` (`System`, `Dark`, or `Light`),
install a complete `Style` for both `Theme::Dark` and `Theme::Light`, and resolve a matching
`ConsolePalette` for the custom-painted Source. Keep fonts shared across themes. This is a small
seam that preserves egui's built-in theme switching and prevents new UI from acquiring colors
through unrelated constants.

The target shape is:

```text
ThemePreference: System | Dark | Light
                 |
                 v
         egui Context::theme() -> Theme::Dark | Theme::Light
                 |                         |
                 +-> Style for stock UI    +-> ConsolePalette for Source painting
                     (Visuals, spacing,         (semantic glyphs, grid, cursor,
                      text styles, widgets)      selection and Source background)
```

Concretely:

1. Replace the single `PALETTE`/`style()` pair with two complete theme definitions. A small
   `ConsoleTheme` value can own a `ConsolePalette` and construct the corresponding egui `Style`.
   Install each style once with `Context::set_style_of` during `Console::new`.
2. Stop forcing `Theme::Dark` in `Console::new`. Default to `ThemePreference::System` when there is
   no saved choice, and expose egui's three-state preference control in View or Settings.
3. At the start of each UI pass, resolve the active `Theme` with `Context::theme()`, select that
   theme's `ConsolePalette`, and pass the palette into `cell_visuals`, `sector_line`, and the Source
   panel/frame. Custom painting should never read a process-global palette.
4. Let ordinary egui widgets read `ui.style()`/`ui.visuals()`. Use `ui.scope` only for deliberate,
   local exceptions. Keep domain-semantic colors such as Function, Note, Bang, cursor bloom, and
   grid lines in `ConsolePalette`; egui's generic widget states cannot express those roles.
5. Install Monaspace once with `Context::set_fonts`. Put typography sizes and named text roles in
   both egui styles, but do not duplicate the actual font data per theme.

This is deliberately not a general theme engine or user-editable palette format. Two compiled-in
themes and one preference are enough to establish the seam before the UI grows.

## How egui 0.36.1 models a theme

This repository pins `egui` and `eframe` 0.36.1, so the API details below are specific to 0.36.1.
egui distinguishes the resolved two-value [`Theme`](https://docs.rs/egui/0.36.1/egui/enum.Theme.html)
(`Dark` or `Light`) from the user's three-value
[`ThemePreference`](https://docs.rs/egui/0.36.1/egui/enum.ThemePreference.html) (`Dark`, `Light`, or
`System`). `System` is the default; if system detection yields no value, egui uses a fallback theme,
which defaults to dark. The preference and fallback live in
[`Options`](https://docs.rs/egui/0.36.1/egui/struct.Options.html), alongside one dark and one light
style.

[`Style`](https://docs.rs/egui/0.36.1/egui/style/struct.Style.html) is egui's complete look-and-feel
value. It contains typography mappings, spacing, interaction behavior, animation timing, scrolling,
wrapping, debug options, and [`Visuals`](https://docs.rs/egui/0.36.1/egui/style/struct.Visuals.html).
`Visuals` owns colors and appearance details including selection, widget interaction-state visuals,
panels, windows, menus, popups, text cursors, shadows, corner radii, warning/error colors, and
disabled opacity. Merely changing `Visuals::dark_mode` does not convert a palette; its documentation
says that flag is primarily a summary of the other visual settings.

The context stores separate styles and chooses one from the resolved theme. The 0.36.1
[`Context` API](https://docs.rs/egui/0.36.1/egui/struct.Context.html) provides:

- `set_theme` to set a `ThemePreference` (a `Theme` also converts into a fixed preference);
- `theme` and `system_theme` to read the resolved and detected themes;
- `set_style_of`, `style_mut_of`, and `style_of` for a specific dark or light style;
- `all_styles_mut` for changes that must be identical in both styles; and
- `global_style_mut`/`set_global_style` for only the currently active style.

The distinction matters when following the system: mutating only the active global style leaves the
other style unchanged when the OS switches. Stable cross-theme rules such as spacing should
therefore be applied with `all_styles_mut` or included when constructing both complete styles;
theme-specific colors belong in `set_style_of`.

egui includes [`global_theme_preference_buttons`](https://docs.rs/egui/0.36.1/egui/widgets/fn.global_theme_preference_buttons.html),
which presents System/Dark/Light choices, and
[`global_theme_preference_switch`](https://docs.rs/egui/0.36.1/egui/widgets/fn.global_theme_preference_switch.html),
which is a compact dark/light toggle. The three-state buttons retain system-following behavior; the
two-state switch resolves to a fixed light or dark preference.

## Global changes, local changes, and custom widgets

New windows, panels, menus, and popups start from the context's active global style. A `Ui` may have
its own inherited style: [`Ui::style_mut`](https://docs.rs/egui/0.36.1/egui/struct.Ui.html#method.style_mut),
`spacing_mut`, and `visuals_mut` affect that UI and subsequent children, using clone-on-write rather
than mutating the context. [`Ui::scope`](https://docs.rs/egui/0.36.1/egui/struct.Ui.html#method.scope)
creates a child UI for temporary overrides that end with the closure. Prefer a scope for an
intentional sub-region; casual mutation of a parent UI can style all later siblings.

Built-in widgets choose among
[`Widgets`](https://docs.rs/egui/0.36.1/egui/style/struct.Widgets.html) states (`noninteractive`,
`inactive`, `hovered`, `active`, and `open`). Custom interactive widgets can use
[`Style::interact`](https://docs.rs/egui/0.36.1/egui/style/struct.Style.html#method.interact) after
allocating a `Response`, so their state follows the active style. Orcvs' Source cells have additional
domain roles—Function, Note, Number, Marker, cursor bloom—which are outside that generic state
model. A theme-specific semantic palette is therefore still appropriate, but it must be selected by
the same resolved `Theme` as egui.

## System preference and persistence

eframe supplies the OS/browser preference to egui as `RawInput::system_theme`; the browser backend
uses the `prefers-color-scheme` media query and updates the value when it changes. `Context::theme()`
therefore follows live system changes while the preference is `System`.
[`Context::system_theme`](https://docs.rs/egui/0.36.1/egui/struct.Context.html#method.system_theme)
returns `None` when the integration cannot determine a preference. egui also synchronizes native
window decorations with its theme preference by default through
[`Options::sync_window_theme`](https://docs.rs/egui/0.36.1/egui/struct.Options.html#structfield.sync_window_theme).

With eframe's `persistence` feature enabled, eframe persists egui memory by default; the
[`App::persist_egui_memory`](https://docs.rs/eframe/0.36.1/eframe/trait.App.html#method.persist_egui_memory)
default is `true`. `ThemePreference` is serializable when egui's `serde` feature is active, and it is
part of serializable `Options`; the actual dark and light styles and detected system theme are
explicitly skipped. Consequently, application startup must always reinstall both compiled-in
styles, while eframe can restore the preference. Persistence is feature-gated in this repository, so
non-persistence builds naturally start from `System` each launch.

eframe restores egui memory before invoking the application creator on both native and web. The
current unconditional `set_theme(Theme::Dark)` in `Console::new` therefore overwrites a restored
preference. Removing that call is required for built-in egui-memory persistence to work. If Orcvs
later stores all user settings in its own persisted settings object, persisting `ThemePreference`
there instead is also valid, but there should be one owner rather than competing values.

## Fonts and text roles

Fonts are configured separately from `Style`. `FontDefinitions` maps font names to font data and
maps font families to ordered fallback lists; [`Context::set_fonts`](https://docs.rs/egui/0.36.1/egui/struct.Context.html#method.set_fonts)
installs them globally. The official
[`FontDefinitions` example](https://docs.rs/egui/0.36.1/egui/struct.FontDefinitions.html) supports TTF
and OTF data and shows that earlier family entries have higher fallback priority. The console's
existing one-time Monaspace installation already follows this model.

Text sizing and roles do belong to `Style`: [`TextStyle`](https://docs.rs/egui/0.36.1/egui/style/enum.TextStyle.html)
is a semantic alias resolved through `Style::text_styles`, and it supports custom named roles with
`TextStyle::Name`. Use these for future UI roles instead of scattering `FontId` sizes. The Source's
cell geometry may still require an explicit monospace `FontId`, because it is part of the custom
renderer rather than ordinary document typography.

## Current gap in Orcvs

`Console::new` installs only `style()` as the dark style and then fixes the preference to dark. The
style begins with `Visuals::dark`, while `PALETTE` is a single dark-only constant. Standard egui
widgets partly use that installed style, but Source rendering does not: `cell_visuals`,
`bloom_colours`, and `sector_line` read `PALETTE` directly, and the central panel explicitly fills
itself from `PALETTE.source`. Enabling egui's theme buttons alone would therefore produce a mixed UI:
stock widgets could switch while the main Source remained dark.

The console has already made one useful separation: semantic Source colors live in
`ConsolePalette`, while `style()` maps a subset into egui `Visuals`. Preserve that split, make it
per-theme, and make the active theme an explicit input to custom painting. This keeps future panels
and controls aligned with egui while retaining the Source's domain-specific visual language.

## Limitations and version notes

- egui themes are dark/light pairs, not named arbitrary themes. More than two branded schemes would
  require an application-level scheme selector in addition to `ThemePreference`.
- egui does not derive a light palette from a dark palette. Both styles and both semantic palettes
  need deliberate values and contrast review; `dark_mode` alone is insufficient.
- Context-level style changes affect newly created UIs and subsequent popups/menus. Existing local
  UI overrides can continue to differ until reset or recreated, which is another reason to keep
  overrides scoped.
- Fonts are global rather than selected by dark/light theme. Theme-dependent typefaces would need
  explicit font reconfiguration and are not recommended here.
- egui is under active development. Re-check `ThemePreference`, `Options`, `Context`, and widget
  helper APIs when upgrading from the repository's pinned 0.36.1 rather than assuming current docs
  apply unchanged.

## Primary source index

- [egui 0.36.1 `Theme` API](https://docs.rs/egui/0.36.1/egui/enum.Theme.html)
- [egui 0.36.1 `ThemePreference` API](https://docs.rs/egui/0.36.1/egui/enum.ThemePreference.html)
- [egui 0.36.1 `Options` API](https://docs.rs/egui/0.36.1/egui/struct.Options.html)
- [egui 0.36.1 `Context` API](https://docs.rs/egui/0.36.1/egui/struct.Context.html)
- [egui 0.36.1 `Style` API](https://docs.rs/egui/0.36.1/egui/style/struct.Style.html)
- [egui 0.36.1 `Visuals` API](https://docs.rs/egui/0.36.1/egui/style/struct.Visuals.html)
- [egui 0.36.1 `Ui` API](https://docs.rs/egui/0.36.1/egui/struct.Ui.html)
- [egui 0.36.1 fonts API](https://docs.rs/egui/0.36.1/egui/struct.FontDefinitions.html)
- [eframe 0.36.1 `CreationContext` API](https://docs.rs/eframe/0.36.1/eframe/struct.CreationContext.html)
- [eframe 0.36.1 `App` persistence API](https://docs.rs/eframe/0.36.1/eframe/trait.App.html)
- [egui 0.36.1 theme implementation](https://docs.rs/egui/0.36.1/src/egui/memory/theme.rs.html)
- [egui 0.36.1 style/context implementation](https://docs.rs/egui/0.36.1/src/egui/context.rs.html)
- [eframe 0.36.1 web system-theme implementation](https://docs.rs/eframe/0.36.1/src/eframe/web/mod.rs.html)
- [eframe 0.36.1 native integration memory restore](https://docs.rs/eframe/0.36.1/src/eframe/native/winit_integration.rs.html)
- [eframe 0.36.1 web integration memory restore](https://docs.rs/eframe/0.36.1/src/eframe/web/app_runner.rs.html)
