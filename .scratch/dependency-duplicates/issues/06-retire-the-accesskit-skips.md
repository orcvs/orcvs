# 06 — Retire the AccessKit skips once egui and kittest align on one accesskit_consumer

**What to build:** Once egui's `accesskit_winit` is on 0.34 or later, which lines every platform adapter up on `accesskit_consumer ^0.39` and `hashbrown ^0.17`, and `kittest` has a release on the same `accesskit_consumer`, the older AccessKit copies leave the graph and their skip entries come out of `deny.toml`.

Waiting on: an egui/eframe release carrying the AccessKit bump (`emilk/egui#8496` is open as of 2026-09-25) and a `kittest` release after 0.4.0.

**Blocked by:** None — 05 is resolved; the wait is upstream (see below).

**Status:** needs-triage

- [ ] The console depends on eframe and egui_kittest releases that resolve a single `accesskit_consumer`.
- [ ] The `accesskit_consumer` and `hashbrown` skip entries are removed.
- [ ] `cargo deny --locked --all-features check bans` reports no duplicate for those crates and no unused-skip warning.

## Comments

### Issue audit against d3fd1b27 — 2026-10-01

`05` is resolved, so `Blocked by:` now names the upstream wait instead. `emilk/egui#8496` is still
open, `kittest` is still 0.4.0, and the skips are at `deny.toml:69-71`.
