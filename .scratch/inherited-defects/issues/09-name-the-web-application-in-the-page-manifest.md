# 09 — Name the web application in the page manifest

**What to fix:** `manifest.json` still names the `eframe` template. A user who installs the web
application gets an icon labelled "egui Template PWA".

**Status:** resolved

- [x] The manifest names Orcvs.
- [x] The installed application shows that name.
- [x] The short name is consistent with the full name.

## Comments

`shell/assets/manifest.json:2-3`:

```json
"name": "egui Template PWA",
"short_name": "egui-template-pwa",
```

The file is a template leftover. `main` holds the same two lines in `console/assets/manifest.json`.

Two other names describe the same application, and all three must agree. `shell/src/main.rs:50`
sets the native window title to `"[ o r c v s ]"`. `shell/index.html:10` sets the page title, which
the crate-split branch changed from `console` to `orcvs`.

Choose one spelling for the user-facing name and use it in all three files.
