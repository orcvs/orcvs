# 02: One Sector Seam spacing for Tab and the painted seams

**What to build:** Tab and Shift Tab, and the Sector Seams the console paints, read the Sector Seam spacing from one place. Do this before anything can change the spacing while the console runs, such as a viewer setting for it.

**Blocked by:** None. It can start at any time, and it has to land before any runtime spacing setting.

**Status:** needs-triage

## Background

There are two copies of the spacing:
- The Editor (`orcvs/src/editor.rs`) is built with `Editor::new(grid, sector_seam_spacing)`. It keeps that copy for Tab and Shift Tab.
- `Orcvs` builds the Editor from `opts.sector_seam_spacing` once, in `Orcvs::from_playback` (`orcvs/src/app.rs`).
- `Orcvs::render_frame` reads `opts.sector_seam_spacing` again on every frame and passes it to the Render Frame, which paints the Sector Seams from it.

Shipped code never changes `opts.sector_seam_spacing` after `Opts::new()`, so the two copies agree today. Code that changed it at runtime would move the painted seams but not where Tab lands, and nothing would report an error.

This was left as it is on purpose in the Editor extraction (PR #422, review thread on `orcvs/src/app.rs`), because no setting exists yet.

## Options

- **Render Frame reads the Editor's spacing.** `Orcvs::render_frame` takes the spacing from the Editor, and `Opts` stops carrying it, or carries only the start-up default. A spacing setting then calls one Editor method.
- **Pass the spacing to `Editor::handle` on each event.** `Opts` stays the only copy, and the Editor stops storing it. This makes `handle`'s interface wider by one parameter that every call passes.

## Acceptance criteria

- [ ] The spacing that Tab and Shift Tab step by and the spacing the Sector Seams are painted at come from one value. No path can change one without the other.
- [ ] A test changes the spacing, then checks that Tab lands on the first Cell of a painted Sector.
- [ ] The Sector tests in `orcvs/src/editor.rs` still state their spacing directly, without reaching into `Opts`.
