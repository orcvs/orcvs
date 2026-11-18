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

Adopt the stack [the research](../research/typescript-gates.md) recommends as the strictest
available today, every devDependency pinned exactly and installed with `npm ci`:

- **Types.** `typescript` 6.0.3 and `@astrojs/check` 0.9.12, running `astro check` over the `.astro`
  pages and the `.ts` files. Stay on TypeScript 6 until TS 7.1 ships a compiler API: TS 7.0 has
  none, so `@astrojs/check` (peer `^5 || ^6`) and typescript-eslint (peer `<6.1.0`) both need 6.x.
  `site/tsconfig.json` extends `astro/tsconfigs/strictest` and adds
  `noPropertyAccessFromIndexSignature`, `erasableSyntaxOnly`, `noUncheckedSideEffectImports` and
  `checkJs` (the research's proposed tsconfig). The plain JS gets `checkJs` through its own
  tsconfigs, run with the site's `tsc`: `synth/web/synth-host.js` on the DOM lib,
  `synth/web/synth-worklet.js` on `@types/audioworklet` 0.0.102, and `console/assets/sw.js` typed
  as a service worker.
- **Lint.** ESLint 10.15.0 with `@eslint/js` 10.0.1 and `typescript-eslint` 8.73.0
  (`strictTypeChecked` + `stylisticTypeChecked`, `projectService`), plus `eslint-plugin-astro`
  3.2.1, `eslint-plugin-playwright` 2.12.2, `eslint-plugin-unicorn` 80.0.0 and
  `eslint-plugin-regexp` 3.3.3, run with `--max-warnings=0`.
- **Format.** Prettier 3.10.1 with `prettier-plugin-astro` 1.1.2. Format the existing files in a
  commit of their own, listed in `.git-blame-ignore-revs`, so later diffs stay readable.
- **Shell.** ShellCheck 0.11.0 (`--severity=style --enable=all`) and shfmt 3.14.1, pinned through
  mise's aqua backend as `actionlint` and `zizmor` are.
- **Unused code.** knip 6.45.1 for unused files, exports and dependencies.
- **npm audit.** `npm audit` and `npm audit signatures` in their own task: `mise run audit_deps` is
  Cargo-only, so passing it says nothing about npm.

Wire each check into a `mise` task and the pull-request tier of `.github/workflows/site.yml`, and
list them under the `site/` bullet of `CLAUDE.md`'s Verification section and in `docs/tooling.md`.
`site.yml`'s `paths:` filter watches neither `synth/web/**` nor `console/assets/**`, so a change to
the plain JS alone would skip the new gates until those paths are added.

The strict config raises a known finding at `site/src/console/small.ts:68`: `if (started)
fail(reason)` reports TS2801, a false positive. `started` is a module-level `let` that `fail()`
resets to `undefined`; when two presses share one rejected promise, the first catch fails and
clears it, and the second sees `undefined` and skips a second notice. TypeScript keeps its
post-`??=` narrowing across the `await`, so it misreads the guard. Rewrite the guard so the checker
can follow it, for example by comparing against the press's own promise, rather than suppressing
the diagnostic.

Out of scope: `scripts/*.ts` and `scripts/tests/`, which the roadmap and bench tooling own, and
`site/prototype-console-site/`, a checked-in build. Decide them in a follow-up if the same tool
should cover them.

**Blocked by:** None (can start immediately).

**Status:** needs-triage

- [ ] Prettier 3.10.1 and `prettier-plugin-astro` 1.1.2 are pinned with a config, and the
      formatted files land in their own commit, listed in `.git-blame-ignore-revs`.
- [ ] `site/tsconfig.json` extends `astro/tsconfigs/strictest` with
      `noPropertyAccessFromIndexSignature`, `erasableSyntaxOnly`, `noUncheckedSideEffectImports`
      and `checkJs`, and `astro check` passes with `typescript` 6.0.3 and `@astrojs/check` 0.9.12
      pinned.
- [ ] `synth/web/` and `console/assets/sw.js` type-check under `checkJs` with the site's `tsc`.
- [ ] ESLint passes with no warnings on `typescript-eslint`'s `strictTypeChecked` and
      `stylisticTypeChecked` plus the Astro, Playwright, unicorn and regexp plugins at the pins
      above.
- [ ] `small.ts:68` is rewritten so TS2801 no longer fires, without a suppression.
- [ ] `shellcheck --severity=style --enable=all` and `shfmt -d` cover `site/scripts/**/*.sh`, both
      pinned in `mise.toml`.
- [ ] knip passes on `site/`.
- [ ] `npm audit` and `npm audit signatures` run as a `mise` task and in CI.
- [ ] `site.yml`'s `paths:` filter includes `synth/web/**` and `console/assets/**`.
- [ ] Each check is a `mise` task and runs in `site.yml`'s pull-request tier; `bash
      scripts/check-tooling-contract.sh`, `actionlint` and `zizmor --offline .github/workflows`
      pass.
- [ ] `CLAUDE.md` and `docs/tooling.md` name the new gates.
- [ ] The `dependency` gate in `CLAUDE.md` (`mise run audit_deps`) passes; record why each new
      devDependency is worth adding.
