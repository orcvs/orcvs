# 33 — Gate the site's TypeScript and JavaScript

**What to build:** The TypeScript and JavaScript the site ships and tests (`site/src/`,
`site/tests/`, `site/astro.config.mjs`), plus the browser code beside it (`synth/web/`,
`console/assets/sw.js`), get the format, type and lint checks the Rust has. Today none of
them runs:

- **Format.** Nothing formats or checks this code, and no Prettier config or dependency exists.
  `npx prettier@3 --check 'src/**/*.ts' 'tests/**/*.ts'` in `site/` flags 12 files, so the code
  already drifts from any one style.
- **Types.** `typescript` is not a dependency. `astro build` strips types without checking them,
  so a type error in `site/src/console/*.ts` or a Playwright spec ships or passes unseen.
- **Lint.** No linter runs on this code.
- **Shell.** `site/scripts/console.sh`, which every Cloudflare deploy runs as `prebuild`, gets no
  `shellcheck` from any gate.

Pick one formatter (Prettier, or Biome, which also lints), with its config and a pinned
devDependency, and format the existing files in a commit of their own so later diffs stay
readable. Add `typescript` and `@astrojs/check` and run `astro check`. Wire each check into a
`mise` task and the pull-request tier of `.github/workflows/site.yml`, and list them under the
`site/` bullet of `CLAUDE.md`'s Verification section and in `docs/tooling.md`.

Out of scope: `scripts/*.ts` and `scripts/tests/`, which the roadmap and bench tooling own, and
`site/prototype-console-site/`, a checked-in build. Decide them in a follow-up if the same tool
should cover them.

**Blocked by:** None (can start immediately).

**Status:** needs-triage

- [ ] A formatter config and pinned devDependency exist, and the formatted files land in their own
      commit.
- [ ] `astro check` passes, with `typescript` pinned in `site/package.json`.
- [ ] A linter runs on the same files, or the chosen formatter's lint mode does.
- [ ] `shellcheck` covers `site/scripts/*.sh`.
- [ ] Each check is a `mise` task and runs in `site.yml`'s pull-request tier; `bash
      scripts/check-tooling-contract.sh`, `actionlint` and `zizmor --offline .github/workflows`
      pass.
- [ ] `CLAUDE.md` and `docs/tooling.md` name the new gates.
- [ ] The `dependency` gate in `CLAUDE.md` (`mise run audit_deps`) passes; record why each new
      devDependency is worth adding.
