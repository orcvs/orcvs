# 05 — Show the Cursor and Region on the Overview

**What to build:** The performer can find their Cursor and Region on the Overview when either is off screen. The Cursor's Cell and the Region are marked over the picture in their existing Theme colours, drawn as overlays so a Cursor move or Region change rebuilds no picture. See `../spec.md` (Overlays).

**Blocked by:** 02 — Show the Overview.

**Status:** ready-for-agent

- [ ] Built test-first at the Shape level: the Region mark covers the Region's Cells and the Cursor mark the Cursor's Cell at their predicted rects, drawn over the picture and under the Source View outline when ticket 03 has landed.
- [ ] They reuse the Cursor and Region Theme keys; no new key.
- [ ] Moving the Cursor or changing the Region does not rebuild the picture.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass; `mise run check_wasm` passes.
