# 03 — Retire the objc2 0.5 skips once AccessKit's macOS adapter moves to objc2 0.6

**What to build:** Once neither winit nor `accesskit_macos` requires `objc2` 0.5, the older `objc2`, `objc2-foundation`, `objc2-app-kit` and `block2` copies leave the macOS graph, and their skip entries come out of `deny.toml`.

Waiting on: an `accesskit_macos` release on `objc2` 0.6 that the eframe release from 02 brings in. As of 2026-09-25, `accesskit_macos` 0.27.0 still requires `objc2 ^0.5.1`.

**Blocked by:** 02

**Status:** needs-triage

- [ ] The four `objc2`-family skip entries are removed.
- [ ] `cargo deny --locked --all-features check bans` reports no duplicate for those crates and no unused-skip warning.

## Comments

### Independent implementation audit — 2026-09-29

Still open: the locked accesskit_macos 0.26.3 and winit 0.30.13 both require objc2 0.5.2;
`deny.toml:47-50` retains all four skips. Shortened the blocker to its issue reference:
the roadmap parser treated the version numbers in the old title as nonexistent issues 30
and 31. Issue 02 is the actual local blocker. The dated upstream-release observation above
is historical; this audit verifies the locked graph, not newer upstream releases.
