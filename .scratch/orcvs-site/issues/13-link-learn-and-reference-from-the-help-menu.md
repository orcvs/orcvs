# 13 — Link Learn and Reference from the Help menu

**What to build:** The console's Help menu carries permanent links to the site's Learn and
Reference pages, `https://orcvs.com/learn/` and `https://orcvs.com/reference/`, on native and on
the web. Toby decided this on 2026-10-26: the site is the console, and Learn and Reference are
reached from its Help menu.

Each link opens in a new browser tab on the web, so the open console and its Source stay where
they are, and in the system browser on native. Native runs the system's opener (`open`,
`xdg-open` or `explorer`) itself rather than enabling eframe's `links` feature, whose `webbrowser`
adds 27 crates, `url` and the ICU tables beneath it, to parse two fixed links.

**Blocked by:** None

**Status:** done

- [x] `Help → Learn` opens `https://orcvs.com/learn/`: in a new tab on the web, in the system browser on native.
- [x] `Help → Reference` opens `https://orcvs.com/reference/` the same way.
- [x] No crate enters `Cargo.lock`.
