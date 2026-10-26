# 03 — Build the site with the console at /play/

**What to build:** `site/` holds an Astro + Starlight site, versions pinned in `site/package.json`
and its lockfile, with a home page that states the language is pre-release, an empty `/learn/` and
`/reference/`, and the console at `/play/`. The `mise.toml` Node pin moves to 22.23.4, which Astro
7.4.2 requires. A `mise` task builds the composed output in the spec's order: Trunk with
`--release --public-url <base>/play/ --dist ../site/public/play`, then Astro, then a check that
`public/play/index.html` names `<base>/play/`. A site job in CI runs that task and Playwright smoke
tests against the output served locally. A path filter runs it for site changes without the Rust
gates, and for console changes too. See `../spec.md` (Framework, Routes, Build, CI paths, Look).

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Nothing built is committed; `site/public/play/` is ignored.
- [x] Smoke tests show `/`, `/learn/`, `/reference/` and `/play/` load with no console errors, and
      that `/play/`'s console starts.
- [x] The site uses the console's font and the built-in Themes' colours.
- [x] A site-only pull request runs the site job and skips the Rust compile; a `console/` change runs
      both.
- [x] `node --test scripts/tests/roadmap.test.ts` passes on the new Node pin.
- [x] `bash scripts/check-tooling-contract.sh`, `actionlint` and `zizmor --offline .github/workflows`
      pass, and `mise run audit_deps` is not needed because no Cargo dependency changes.

## Comments

### Shipped, 2026-10-26

The site job is `.github/workflows/site.yml`, path-filtered and not a required check. The task is
`mise run site_build`, then `mise run site_test`. Two things surfaced on the way:

- Release builds of the console had never worked: Trunk's default `wasm-opt` rejects the
  bulk-memory and float-to-int instructions current Rust emits. `console/index.html` now passes the
  feature flags. Debug builds, which every other gate uses, skip `wasm-opt`.
- Astro 7 backgrounds `astro preview` when it detects an agent, so Playwright saw the server exit.
  The config passes `--ignore-lock`, which keeps it in the foreground.

`npm` rather than `pnpm`: it ships with the pinned Node, so the site adds no tool to `mise.toml`.

### Builds split, 2026-10-26

Toby asked for the site build and the app build to be orthogonal. The console now builds on its own
(`mise run console_web_build`, `.github/workflows/console-web.yml`) with relative URLs, and
`site/build.sh` copies that build into `public/play/` instead of compiling it. The base check is
gone: a relative console has no base to disagree with. The site workflow's path filter no longer
names the console or the crates; a console change reaches the site through the `console-web`
artifact.
