# 04 — Unify arboard's windows-sys on 0.52

**What to build:** The Windows build carries one fewer `windows-sys` copy, and the `windows-targets` 0.53 family goes with it. `arboard` 3.6.1 (egui-winit's clipboard) accepts `windows-sys >=0.52, <0.61`, and the lockfile resolved it to 0.60.2 only because cargo picks the highest compatible version. Moving it onto the 0.52 copy that winit 0.30 and glutin 0.32 already require is a lockfile-only change: no manifest changes and no new crates.

The pin doesn't survive a broad `cargo update`, which picks 0.60 again. Until 07 lands, that shows up as returning duplicate warnings; after 07, it fails the audit.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] `Cargo.lock` no longer contains `windows-sys` 0.60, `windows-targets` 0.53 or the `windows_*` 0.53 platform crates, and changes nothing else.
- [ ] `cargo deny --locked --all-features check bans` no longer warns about `windows-targets`, `windows_x86_64_gnu` or `windows_x86_64_msvc`, and reports three `windows-sys` entries.
- [ ] `mise run audit_deps` passes.
