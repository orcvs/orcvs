# 07 — Load a base16 scheme at runtime

**What to build:** A viewer supplies a published base16 scheme and the console paints from it, on both the native and the WASM target, without a new dependency.

**Blocked by:** 06 — Paint the Source from a scheme.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] A base16 scheme parses into the `Scheme` `06` defines. After the parse there is one code path: a loaded scheme and a compiled-in one are the same type and are treated identically everywhere.
- [ ] The parser is hand-written and adds no dependency. A base16 scheme file is a flat `base00: "#1d2021"` map with a `name` and a `system` line; the subset that needs reading is small. `serde_yaml` is archived upstream and is not the easy answer it looks like, and `deny.toml` runs `all-features` across five targets including `wasm32-unknown-unknown`.
- [ ] Malformed input is refused whole and reported, never partly applied. Sixteen well-formed slots or nothing — the rule `SourcePaintSettings::decode` already followed.
- [ ] The native target loads from a file. The WASM target has no filesystem, so it takes pasted text. Both reach the same parser.
- [ ] A loaded scheme survives restart with `persistence` on: its sixteen values are stored, since a loaded scheme has no name the console can resolve later. A build without `persistence` keeps it for the run.
- [ ] A loaded scheme becomes a Theme like any other: it appears in the dark or light picker by its `variant`, or by `base00`'s lightness where it has none, and every named key resolves from its default slot.
- [ ] `08`'s validator runs on load and its report is shown. A scheme that fails the contrast floor is loaded anyway and reported, not refused — the viewer chose it.
- [ ] `mise run check_wasm` passes.

## Comments

**Why a loader at all, given compiled-in schemes exist.** The point of adopting base16 rather than a shape of our own is the several hundred published schemes. Compiled-in schemes prove the template; the loader is what makes the ecosystem reachable without a release.

**Scheme identity.** A compiled-in scheme is stored by name and re-resolved at startup, so improving one reaches every install. A loaded scheme has no name the console can resolve, so its values are stored. Both are the viewer's choice, which is why storing them does not reintroduce the problem ADR 0051 describes — what is forbidden is storing a colour the viewer did not choose.

**2026-09-21 — revised for ADR 0053.** No overrides exist to key by scheme, so that acceptance line is replaced. A loaded scheme is stored as a Theme document, which is the same storage `10` uses for custom Themes.
