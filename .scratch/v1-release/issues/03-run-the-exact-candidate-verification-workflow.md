# 03 — Run the exact-candidate verification workflow

**What to build:** Provide and run one reproducible release-candidate task/workflow against the
nominated commit SHA, publishing authoritative native, persistence, WASM, browser, semantic, and
documentation results without moving the full cost onto every ordinary pull request.

**Blocked by:** inherited-defects/15; orcvs-language-migration/08; property-testing/02; property-testing/03; property-testing/04; property-testing/05; sequence-values/03; sequence-values/05; source-module-depth/07; source-module-depth/08; spatial-tick-planning/03; spatial-tick-planning/04; spatial-tick-planning/05; tick-functions/02; tick-functions/03; tick-functions/04; midi-output-family/03; midi-output-family/04; restyle-egui-console/01; restyle-egui-console/02; v1-release/02.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] The task starts from a clean checkout and records candidate SHA, pinned toolchain, exact
      commands, exit statuses, and authoritative CI or workflow links.
- [ ] `mise run check` passes and the exact SHA is green for Linux and macOS native coverage,
      persistence, both WASM feature builds, and headless Firefox behavior.
- [ ] Every shipped inventory member maps to positive and applicable boundary/failure evidence.
- [ ] Finite domains run exhaustively and each structured property runs at least 256 cases with
      committed regressions.
- [ ] The published result records the merge tier's rustdoc and dependency-policy results by name:
      `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked` and
      `cargo deny --locked check`, each with its exit status. Both already run inside
      `mise run check`; this names them in the record rather than commissioning new work.
- [ ] The published result can be consumed by visual, physical MIDI, benchmark, and final GO/NO-GO
      review without rerunning against a different commit.
