# 02 — Read the web console's settings from its page

**What to build:** ADR 0090's page settings. The web console reads `config.toml` text from the
`orcvs_config` element in `console/index.html`, through the same reader as native, when its
`data-media` query matches or it has none. The shipped page starts a phone, `(max-width: 600px)`,
with 24 by 36 point Cells and the Overview closed.

See `../spec.md`.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] Page text is read by native's reader, and a problem in it is a notice naming the page.
- [ ] The web console starts with the page's settings when the media matches, and the defaults when
      it does not or there is no element.
- [ ] `index.html` holds the phone setting.
