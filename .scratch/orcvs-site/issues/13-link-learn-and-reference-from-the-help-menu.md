# 13 — Link Learn and Reference from the Help menu

**What to build:** The console's Help menu carries permanent links to the site's Learn and
Reference pages, `https://orcvs.com/learn/` and `https://orcvs.com/reference/`, on native and on
the web. Toby decided this on 2026-10-26: the site is the console, and Learn and Reference are
reached from its Help menu.

Each link opens in a new browser tab on the web, so the open console and its Source stay where
they are, and in the system browser on native. Native needs eframe's `links` feature, which
brings in `webbrowser` through egui-winit; without it eframe logs a warning and opens nothing.

**Blocked by:** None

**Status:** done

- [x] `Help → Learn` asks egui to open `https://orcvs.com/learn/` in a new tab.
- [x] `Help → Reference` asks egui to open `https://orcvs.com/reference/` in a new tab.
- [x] Native opens both in the system browser: the console enables eframe's `links` feature with a
      recorded rationale.
