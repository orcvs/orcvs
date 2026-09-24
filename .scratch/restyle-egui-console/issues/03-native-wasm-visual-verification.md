# 03 — Native/WASM visual verification

**What to build:** Capture the nominated candidate on native and WASM at wide and tall viewport
sizes in both built-in Themes, with enough metadata and human review to reproduce and assess the rendered evidence. The release capture workflow (`release-captures`) produces the eight CI captures; a person adds one native macOS capture per Theme and reviews all ten.

**Blocked by:** 01 — Square, centred Source Grid viewport; 02 — Prototype-aligned console palette; v1-release/03 — Run the exact-candidate verification workflow; release-captures/01 — Capture the native console in CI; release-captures/02 — Capture the web console in CI; theming/18 — Correct the Theme record the captures are checked against.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] One dispatched run of the release capture workflow against the exact candidate SHA produces
      eight captures: native and WASM × wide and tall × Okabe–Ito and Orcvs Light.
- [ ] One manual native macOS capture per Theme, on real hardware, is taken from the same SHA.
- [ ] Each capture records SHA, OS or browser, renderer, viewport dimensions, mode (set
      explicitly to Dark or Light, never Follow the OS), Theme, egui zoom (1.0), capture procedure,
      date, and reviewer.
- [ ] The checklist verifies square whole-pixel Cell geometry, the Source View at its top-left
      rest with the two-Cell margin (ADR 0047), each built-in Theme's decided values in
      `console/src/theme.md`, occupied and empty Cells, the Function, Number, Note, Bang, Comment,
      Sequence, fitted Output Portal and invalid-operand diagnostic distinctions, a Region, Sector
      Seams, and the Cursor and its effect area.
- [ ] The exact palette and any remaining prototype differences are reported.

## Comments

**2026-09-24 — rescoped to both built-in Themes, with captures produced in CI.** Orcvs Light ships as an accepted, selectable built-in and the default mode follows the OS, so "the decided palette" now means both Themes and every capture pins its mode. The release decision is eight CI captures from a dispatch-only workflow (`.scratch/release-captures/`), which renders natively through a software GPU driver and in headless Firefox, plus one manual macOS capture per Theme as the real-GPU check. The checklist's "centred" line was replaced: ADR 0047 rests the Source View at the top-left with a margin. Its state list now names what `theme.md` and the contrast gate decide. The light captures in `.scratch/theming/evidence/` came from a patched build and do not count as candidate evidence. Blockers 01 and 02 keep their original titles as history; both are resolved.
