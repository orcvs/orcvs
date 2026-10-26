# 01 — Open a small console by message

**What to build:** ADR 0085's small console. `/play/?small` opens Linked on an empty Source with
Playback stopped and without the menu bar. It accepts `{ type: "orcvs.open", source }` posted from
its own origin: a Source the Source File reader accepts replaces the open one with Playback
stopped; a refused one raises a notice and leaves the open Source in place; anything else is
ignored.

See `../spec.md`.

**Blocked by:** None

**Status:** resolved

- [x] `?small` opens a Linked, empty, stopped console with no menu bar.
- [x] An accepted message replaces the Source and leaves Playback stopped.
- [x] A refused Source raises a notice; a foreign origin or another type changes nothing.
- [x] `mise run check_wasm` passes.
