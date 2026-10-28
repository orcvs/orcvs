# 02 — Read the web console's settings from its page

**What to build:** ADR 0090's page settings. The web console reads `config.toml` text from the
`orcvs_config` element in `console/index.html`, through the same reader as native, when its
`data-media` query matches or it has none. The shipped page starts a phone, `(max-width: 600px)`,
with 24 by 36 point Cells and the Overview closed.

See `../spec.md`.

**Blocked by:** 01

**Status:** resolved

- [x] Page text is read by native's reader, and a problem in it is a notice naming the page.
- [x] The web console starts with the page's settings when the media matches, and the defaults when
      it does not or there is no element.
- [x] `index.html` holds the phone setting.

## Comments

- The reader (`parse`) leaves the native-only module, so the web reads page text through it.
  `Config::from_page` names the page's `orcvs_config` element in a notice.
- `web_startup::page_settings` reads the element and answers its `data-media` with
  `Window::match_media`, which needs web-sys's `MediaQueryList` binding and no new dependency.
- `the_page_settings_are_read_as_the_file_is_and_named_in_a_notice` covers the reader. The DOM read
  is checked by building the web console and opening it at phone and desktop widths.
