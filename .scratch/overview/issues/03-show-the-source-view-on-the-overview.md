# 03 — Show the Source View on the Overview

**What to build:** The Overview outlines the part of the Grid the Source View shows. The outline moves with every Pan — wheel, middle-drag, Alt-drag, and Cursor follow — shrinks as the performer Zooms in, is clipped to the Overview on an axis with nowhere to Pan, and is absent when neither axis can Pan. A new Theme key sets its colour and width. See `../spec.md` (Overlays, Theme).

**Blocked by:** 02 — Show the Overview.

**Status:** ready-for-agent

- [ ] Built test-first at the Shape level: the outline is the Source View's visible region in Source points, offset by the margin and scaled to the Overview — not the visible Positions, which carry an extra Cell each side — at its predicted rect after a Pan and after a Zoom; clipped on an axis with nowhere to Pan; absent when neither axis can Pan.
- [ ] The outline key is added to the Theme's style catalogue, the Theme schema and both built-in Themes within ADR 0053's chrome width bounds; a custom Theme missing it is refused as any missing key is.
- [ ] Nothing on the Overview animates.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass; `mise run check_wasm` passes.
