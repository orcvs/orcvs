# 05 — Skip the duplicates upstream releases still hold

**What to build:** cargo-deny reports no duplicate-version warnings. Every duplicate left after 04 is held by an upstream crate's latest release, so each gets a skip entry that states the constraint:

- `windows-sys` 0.52: glutin 0.32 (latest 0.32.3 requires `^0.52`) and winit 0.30.
- `windows-sys` 0.59: rustix 0.38 in the winit 0.30 Wayland stack. It retires with 02.
- `accesskit_consumer` 0.35 and 0.36, and `hashbrown` 0.16: the AccessKit adapters egui's `accesskit_winit` pins (`accesskit_windows` 0.32, `accesskit_atspi_common` 0.18, `accesskit_macos` 0.26), and `kittest` 0.4.0 (latest) requires `accesskit_consumer ^0.35`. They retire with 06.
- `syn` 2: most proc macros in the graph (`zerocopy-derive`, `enumn`, `tracing-attributes`, `zbus`) are still on `syn` 2 while eframe and others use `syn` 3. This is an ecosystem-wide migration with no retirement ticket.
- `getrandom` 0.3: `ahash` 0.8.12 (via epaint) and `rand_core` 0.9 (via proptest) require it in their latest releases, while `tempfile` and `jobserver` use 0.4.

**Blocked by:** 04 — Unify arboard's windows-sys on 0.52.

**Status:** ready-for-agent

- [ ] Each entry names the held version and the upstream crate that holds it.
- [ ] `cargo deny --locked --all-features check bans` reports no duplicate warnings and no unused-skip warning.
- [ ] `mise run audit_deps` passes.
