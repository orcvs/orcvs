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

**Status:** ready-for-agent

- [ ] Nothing built is committed; `site/public/play/` is ignored.
- [ ] Smoke tests show `/`, `/learn/`, `/reference/` and `/play/` load with no console errors, and
      that `/play/`'s console starts.
- [ ] The site uses the console's font and the built-in Themes' colours.
- [ ] A site-only pull request runs the site job and skips the Rust compile; a `console/` change runs
      both.
- [ ] `node --test scripts/tests/roadmap.test.ts` passes on the new Node pin.
- [ ] `bash scripts/check-tooling-contract.sh`, `actionlint` and `zizmor --offline .github/workflows`
      pass, and `mise run audit_deps` is not needed because no Cargo dependency changes.
