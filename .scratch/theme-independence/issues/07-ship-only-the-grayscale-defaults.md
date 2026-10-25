# 07 — Ship only the grayscale defaults

**What to build:** The console ships two built-in Themes, Grayscale Dark and Grayscale Light. Okabe–Ito, Orcvs Light, My Grayscale Dark and My Grayscale Light are deleted: their definitions, reserved identities, registry entries, Theme record sections and evidence, record tests, the Okabe–Ito copy example and its test. The release-capture workflow and `mise` task capture the two defaults. ADR 0053's text naming the retired Themes as shipped is amended. A saved selection naming a retired identity falls back to the appearance's default. A custom Theme inheriting from one is refused with a notice naming the setting.

**Blocked by:** 02, 03, 04, 05 and 06.

**Status:** ready-for-agent

- [ ] The registry holds exactly two built-ins, and the Theme menu offers them plus any custom Themes.
- [ ] `git grep -i -E 'okabe|orcvs.light|my.grayscale'` outside `.scratch/` and ADR history finds nothing.
- [ ] A test shows a saved selection of `okabe-ito` falling back to Grayscale Dark, and a document inheriting from `okabe-ito` being refused.
- [ ] The Theme record's Shipped Themes table lists two Themes, and the record tests pass.
- [ ] `cursor-foreground/03` is narrowed to the two remaining built-ins.
- [ ] `cargo fmt --all -- --check`, the console clippy and nextest gates, the `--no-default-features` arm, `mise run check_wasm`, `mise run check_release_capture`, `bash scripts/check-tooling-contract.sh`, `actionlint` and `zizmor --offline .github/workflows` pass.
