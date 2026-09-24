# 01 — Skip the duplicates winit 0.30 and AccessKit's macOS adapter keep in the graph

**What to build:** cargo-deny's duplicate-version warnings stop reporting the crates the pinned eframe 0.36 stack keeps at old versions, so the warnings left in the audit are the ones worth reading. eframe 0.36 pins winit 0.30. No lockfile update can lift the older copies, and dropping the eframe `wayland` feature would cost native Wayland. The `[bans]` skip list in `deny.toml` records each one in two reason groups:

- winit 0.30 alone (its Linux Wayland stack and its macOS `core-graphics` binding): `thiserror`, `thiserror-impl`, `smithay-client-toolkit`, `calloop`, `calloop-wayland-source`, `rustix`, `linux-raw-sys`, `core-foundation`, `bitflags`.
- winit 0.30 and `accesskit_macos` (still on `objc2 ^0.5.1` at 0.27.0): `objc2`, `objc2-foundation`, `objc2-app-kit`, `block2`.

Each entry names the old release line (`rustix@0.38`), not an exact version, so a patch release within that line still matches it after `cargo update`. A version outside the skipped lines is a new duplicate, which fails the audit once 07 sets `multiple-versions = "deny"`.

**Blocked by:** None (can start immediately).

**Status:** resolved

- [x] The skip list names all 13 old release lines, each with a reason that states the upstream constraint.
- [x] `cargo deny --locked --all-features check bans` reports 8 duplicate warnings (down from 21), with no unused-skip warning.
- [x] `mise run audit_deps` passes.
