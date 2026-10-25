# 07 — Ship only the grayscale defaults

**What to build:** The console ships two built-in Themes, Grayscale Dark and Grayscale Light. Okabe–Ito, Orcvs Light, My Grayscale Dark and My Grayscale Light are deleted: their definitions, reserved identities, registry entries, Theme record sections and evidence, record tests, the Okabe–Ito copy example and its test. The release-capture workflow and `mise` task capture the two defaults. ADR 0053's text naming the retired Themes as shipped is amended. A saved selection naming a retired identity falls back to the appearance's default. A custom Theme inheriting from one is refused with a notice naming the setting.

**Blocked by:** 02, 03, 04, 05 and 06.

**Status:** resolved

- [x] The registry holds exactly two built-ins, and the Theme menu offers them plus any custom Themes.
- [x] `git grep -i -E 'okabe|orcvs.light|my.grayscale'` outside `.scratch/` and ADR history finds nothing, except the deliberate hits the Comments name.
- [x] A test shows a saved selection of `okabe-ito` falling back to Grayscale Dark, and a document inheriting from `okabe-ito` being refused.
- [x] The Theme record's Shipped Themes table lists two Themes, and the record tests pass.
- [x] `cursor-foreground/03` is narrowed to the two remaining built-ins.
- [x] `cargo fmt --all -- --check`, the console clippy and nextest gates, the `--no-default-features` arm, `mise run check_wasm`, `mise run check_release_capture`, `bash scripts/check-tooling-contract.sh`, `actionlint` and `zizmor --offline .github/workflows` pass. (Run here: every one but `actionlint`, `zizmor` and `check_wasm`'s `trunk` builds, which are not installed; the `mise` tasks ran as their commands. See Comments.)

## Comments

The registry's built-ins are `[grayscale_dark(), grayscale_light()]`. A reserved identity is one the registry's built-ins hold: `resolve` reads it from the built-ins it is given and discovery from `ThemeRegistry::is_built_in`, so no second list names them. The four retired definitions, their identities, the Okabe–Ito copy example and its test, and the per-Theme record tests are deleted; the record check moved into `theme::record::assert_matches_its_table`, which `theme::tests::every_built_in_defines_every_key_at_its_table` and `style::tests::every_built_in_chrome_matches_its_record` run over every built-in. `contrast::accepted_failures` returns an empty list for any identity. `theme.md` keeps the Theme model, the Input Portal and Locked roles and the measurement method, and records the two defaults alone; its Shipped Themes table lists two Themes. ADR 0053 carries a 2026-10-25 amendment and notes at the amendments that named the retired Themes. `cursor-foreground/03` is narrowed.

Tests: `theme_selection::tests::a_selection_of_a_retired_built_in_falls_back_to_the_default_and_is_kept` (`okabe-ito` and `orcvs-light`) and `theme_registry::discovery_tests::a_theme_file_inheriting_a_retired_built_in_is_refused` (`inherits = "okabe-ito"`, refused with a notice naming it).

The grep criterion has three deliberate hits outside `.scratch/` and ADR history. The two tests above must name a retired identity to prove the fallback and the refusal. `console/prototypes/syntax-highlighting/source-paint-prototype.html` names the published Okabe–Ito palette as one of the prototype's own palettes, not the built-in Theme. `examples/tracker.md`'s "Orcvs lights" matches `orcvs.light` by accident.

`mise` is not installed in this environment, so `check_wasm` and `check_release_capture` ran as their clippy lines directly; `trunk`, `actionlint` and `zizmor` are not installed, and the workflow change is a comment.

**2026-10-25 — review.** `contrast::accepted_failures`, `AcceptedFailure` and `ContrastResult::accepted` are deleted: with every list empty they could never answer true. The colour-vision gate's neutral-pair exemption is now backed by `theme::tests::roles_whose_glyphs_read_alike_differ_in_fill_or_border`, which asks the question of every viewer: two glyphs closer than `CONFUSION_FLOOR` on the Grid background must differ in fill or drawn border (it caught a Number grey moved to within 2 of Note's in a trial). Discovery records a refusal for any identity, a built-in's included, since `ThemeRegistry::select` answers the built-in first. The unknown-parent notice names `inherits`. The Orcvs Light evidence under `.scratch/theming/evidence/` is deleted, `schema.md`'s sample and reserved identities name the defaults, and the workbench's extractor reads the one-line registry and re-embeds the two built-ins.
