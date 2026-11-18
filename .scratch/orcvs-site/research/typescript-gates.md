# Research: the strictest practical TypeScript and JavaScript gates for the site

Input to [orcvs-site/33](../issues/33-gate-the-sites-typescript-and-javascript.md). Researched
2026-11-18. Versions come from the npm registry (`npm view <pkg> version time`) on that date
unless a line says otherwise. **[inferred]** marks a conclusion I drew rather than read in a source.
**[measured]** marks something I ran against this checkout from a scratch directory, without
installing anything into the repository.

## Recommendation

Use **TypeScript 6.0.3**, not 7. **`@astrojs/check` 0.9.12** runs `astro check` over the `.astro`
pages and the `.ts` files, using Astro's `strictest` preset plus four more flags (see the proposed
tsconfig below). Plain `tsc -p` with `checkJs` covers `synth/web/` and `console/assets/sw.js`, with
**`@types/audioworklet` 0.0.102** for the worklet. Lint with **ESLint 10.15.0** and
**`typescript-eslint` 8.73.0** (`strictTypeChecked` + `stylisticTypeChecked`, `projectService`),
plus **`eslint-plugin-astro` 3.2.1**, **`eslint-plugin-playwright` 2.12.2**,
**`eslint-plugin-unicorn` 80.0.0** and **`eslint-plugin-regexp` 3.3.3**. Format with
**Prettier 3.10.1** and **`prettier-plugin-astro` 1.1.2**. Check the shell scripts with
**ShellCheck 0.11.0** and **shfmt 3.14.1**, pinned through mise's aqua backend the way `actionlint`
and `zizmor` already are. Run **knip 6.45.1** for unused files, exports and dependencies. Pin every
devDependency exactly and install with `npm ci` against the committed lockfile. The ESLint stack is
the strictest coverage available today: all 61 typescript-eslint type-aware rules, linting of
`.astro` files, and Playwright and regexp rule sets that no Rust-based linter carries natively. On
about 1,600 lines its speed cost is seconds. TypeScript 7 is blocked for this stack: TS 7.0 ships
no compiler API, so `@astrojs/check` (peer `typescript ^5 || ^6`) and typescript-eslint (peer
`<6.1.0`) both require 6.x. Reconsider Oxlint with tsgolint and oxfmt once TS 7.1 ships an API,
oxfmt reaches 1.0, and Oxlint lints Astro templates.

## Code in scope, as it stands

- `site/src/console/*.ts` (4 files), `site/tests/*.ts` (8), `site/playwright.config.ts`,
  `site/src/content.config.ts`, and `site/astro.config.mjs` (already `// @ts-check`).
- Three `.astro` pages: `src/pages/index.astro` with two `<script is:inline>` blocks, and
  `src/pages/{learn,reference}/index.astro`, each with a bundled `<script>`. A tool that cannot read
  `.astro` therefore leaves part of the shipped code unchecked.
- `synth/web/synth-host.js` (main thread, DOM), `synth/web/synth-worklet.js`
  (`AudioWorkletGlobalScope`), and `console/assets/sw.js` (ES5-style service worker using `var` and
  `function`). `site/public/play/` holds gitignored copies of these, so every tool must exclude it.
- `site/scripts/console.sh` and `site/scripts/tests/console.sh`.
- Out of scope: `site/prototype-console-site/`, which is a checked-in build and must be ignored by
  every tool.

### What a gate finds today [measured]

I ran `typescript@6.0.3`'s `tsc --noEmit` from a scratch directory with a tsconfig whose `include`
named the site's files:

- **Current config** (`astro/tsconfigs/strict`): **1 error**, a false positive. At
  `src/console/small.ts:68`, `if (started) fail(reason)` is TS2801: "This condition will always
  return true since this 'Promise<void>' is always defined." The guard is deliberate: `started` is
  a module-level `let` that `fail()` resets to `undefined`, so when two presses share one rejected
  promise the first catch fails and clears it and the second skips a second notice. TypeScript
  keeps its post-`??=` narrowing across the `await` for a mutable module variable, so it misreads
  the guard. The strict gate will raise it, so the code needs a rewrite the checker can follow (for
  example, comparing against the press's own promise), not a suppression.
- **Proposed strictest config:** **58 errors** in 9 files, 35 of them in `sound-module.ts`. They
  break down as TS2532 ×26 and TS18048 ×8 (`noUncheckedIndexedAccess`), TS4111 ×8
  (`noPropertyAccessFromIndexSignature`: `process.env.CI`, `dataset.source` and similar),
  TS2345 ×9, TS2322 ×4, TS2412 ×2 (`exactOptionalPropertyTypes`), and the TS2801 above. The full
  check took about 9 s, most of it loading Astro's types.
- **`console/assets/sw.js`** with `checkJs` and `lib: ["WebWorker"]`: 14 errors, because `self`
  types as `WorkerGlobalScope` and events as `Event`. It needs the service-worker typing described
  under question 1.
- **ShellCheck 0.9.0** (the local `/usr/bin` copy) with `-S style` on `site/scripts/*.sh` and
  `site/scripts/tests/*.sh`: no findings.
- **Oxlint 1.94.0 + oxlint-tsgolint 7.0.2007 + TypeScript 7.0.2** with `--type-aware
  --type-check`, `-D correctness -D suspicious -D pedantic -D perf`, ran in about 0.2 s. It found
  the same TS2801, and also reported it as `no-misused-promises` and
  `strict-boolean-expressions`. It produced 109 `prefer-readonly-parameter-types` hits (noise for
  DOM code), 28 `require-unicode-regexp`, 15 `no-unsafe-member-access`, 8
  `no-confusing-void-expression`, 8 `no-unsafe-type-assertion`, 4 `no-misused-spread`, and 2
  `unbound-method` in the Playwright specs. This gives a fair preview of what
  `strictTypeChecked` will report too, since the rules have the same names and options
  **[inferred]**.

## 1. Type checking

### TypeScript 6 vs 7 / tsgo

- **TypeScript 7.0** is the Go-native compiler. It shipped 2026-07-08 as the `typescript` package
  itself, providing `tsc`; the latest version is 7.0.2. Per the
  [announcement](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/), "7.0 does
  not ship with an API". Tools that embed TypeScript ("Vue, Astro, Svelte, Volar-based setups") must
  stay on 6.0. 7.1 is expected to bring a new, different API. For side-by-side use, Microsoft
  publishes `@typescript/typescript6`, which provides `tsc6` and the 6.0 API, and can be aliased:
  `"typescript": "npm:@typescript/typescript6@^6.0.2"`,
  `"@typescript/native": "npm:typescript@^7.0.2"`.
  `@typescript/native-preview` is retired in favour of `typescript@next`.
- 7.0 also changes defaults: `strict` is true, `module` is `esnext`, `noUncheckedSideEffectImports`
  is true, `types` is `[]`, `libReplacement` is false, and `rootDir` is `./`. It makes `baseUrl`,
  `moduleResolution: node10/classic`, `esModuleInterop: false` and `target: es5` hard errors. JS
  analysis was reworked to match `.ts` analysis: no `@enum`, no Closure function types, values are
  not usable as types. Astro's base preset uses none of the removed options [measured: read
  `node_modules/astro/tsconfigs/base.json`].
- **Latest 6.x is 6.0.3** (2026-04-16). Astro 7.4.2 itself develops against `typescript ^6.0.3`
  (`astro/package.json` devDependencies).

### `astro check` / `@astrojs/check`

- Astro 7.4.2's `astro check` command loads `@astrojs/check` and `typescript` from the project, and
  errors if either is missing (`node_modules/astro/dist/cli/check/index.js`). Astro 7.4.2 pins
  `@astrojs/check` 0.9.12 as its own devDependency, which is good evidence the two work together.
- `@astrojs/check@0.9.12` (2026-10-21) declares `peerDependencies: { typescript: '^5.0.0 || ^6.0.0' }`
  and depends on `@astrojs/language-server ^2.17.3`. **It does not declare support for TS 7.**
  `@astrojs/language-server` 2.16.12 fixed a crash "when astro check runs against the TypeScript 7
  native compiler" ([release
  note](https://newreleases.io/project/github/withastro/astro/release/@astrojs%2Flanguage-server@2.16.12)).
  I read that as fixing a confusing failure, not as TS 7 support **[inferred]**.
- **Decision:** pin `typescript` 6.0.3. With TS 6 pinned, `astro check` is the one type gate for
  `.astro` and `.ts`. A separate `tsc -p` covers the plain JS outside `site/`.

### Strictest tsconfig flags

[`astro/tsconfigs/strictest`](https://github.com/withastro/astro/blob/main/packages/astro/tsconfigs/strictest.json)
(read from `node_modules`, Astro 7.4.2) extends `strict` and `base`. It already sets:

- from `base`: `verbatimModuleSyntax`, `isolatedModules`, `forceConsistentCasingInFileNames`,
  `allowJs`, `noEmit`, `skipLibCheck`, `moduleResolution: Bundler`
- from `strict`: `strict`
- from `strictest`: `noFallthroughCasesInSwitch`, `noImplicitOverride`, `noImplicitReturns`,
  `noUnusedLocals`, `noUnusedParameters`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`,
  `allowUnreachableCode: false`, `allowUnusedLabels: false`

Worth adding on top ([tsconfig reference](https://www.typescriptlang.org/tsconfig/)):

| Flag | Why |
|---|---|
| `noPropertyAccessFromIndexSignature` | Forces `env["CI"]` and `dataset["source"]`, which marks lookups that may be absent. 8 hits today. typescript-eslint's `dot-notation`, which is in `stylisticTypeChecked`, honours this flag. I recalled that from its rule docs and did not re-read it **[unverified]**. |
| `erasableSyntaxOnly` (TS 5.8+) | Bans enums, namespaces and parameter properties. Keeps site code loadable by Node's type stripping, the same constraint `scripts/*.ts` already lives under. |
| `noUncheckedSideEffectImports` | Errors on a side-effect import that resolves to nothing. It is the TS 7 default, so this makes the TS 6 behaviour match. |
| `checkJs` | Type-checks `astro.config.mjs` and any `.js`/`.mjs` in the project, instead of relying on per-file `// @ts-check`. |
| `noErrorTruncation` (optional) | Full types in messages. Diagnostic only. |

Flags I considered and rejected:

- `isolatedDeclarations` only constrains declaration emit, and this project has `noEmit` and
  publishes no library.
- `useUnknownInCatchVariables` and `strictBuiltinIteratorReturn` are already part of `strict`.
- `noImplicitAny` and the other strict-family flags are covered by `strict`.

### Checking the plain JS (`checkJs` + JSDoc)

- Give each JS target its own small tsconfig with `allowJs`, `checkJs`, `noEmit`, `strict` and the
  strictest flags. Each runs on its own `lib`, because a page, a worklet and a service worker have
  different globals:
  - `synth/web/synth-host.js`: `lib: ["ES2024", "DOM", "DOM.Iterable"]`.
  - `synth/web/synth-worklet.js`: TypeScript ships no AudioWorklet lib. TS 6.0.3's `lib/` has
    `dom`, `webworker`, and `webworker.importscripts`/`iterable` only [measured]. Use
    `lib: ["ES2024"]` plus `types: ["audioworklet"]` from `@types/audioworklet` (0.0.102,
    2026-11-13). That package is generated by Microsoft's TypeScript-DOM-lib-generator, "Types for
    the global scope of Audio Worklets". The two synth files need separate configs, because DOM and
    AudioWorklet globals conflict **[inferred]**.
  - `console/assets/sw.js`: `lib: ["ES2024", "WebWorker"]`, then type `self` as a service worker.
    Either declare it in JSDoc (`/** @type {ServiceWorkerGlobalScope} */ const sw = /** @type {any} */ (self);`)
    or add `@types/serviceworker` (0.0.210, 2026-11-16) in place of the `WebWorker` lib. Without
    one of these, `e.waitUntil`, `e.request` and `self.clients` are 14 errors [measured].
- Run these with the site's pinned `tsc` (`tsc -p ../synth/web/tsconfig.json`) so there is a single
  TypeScript. The tsconfigs can live beside the code, or under `site/` with absolute-style
  `include`s **[inferred]**.
- TS 7 tightens JSDoc semantics. Keeping JSDoc to plain `@type`/`@param` with TS-syntax types
  makes the later move cheap **[inferred]**.

## 2. Linting

### typescript-eslint (recommended)

- `typescript-eslint` 8.73.0 (2026-11-16). Peers: `eslint ^8.57 || ^9 || ^10`,
  `typescript >=4.8.4 <6.1.0` ([dependency
  versions](https://typescript-eslint.io/users/dependency-versions/)). No TS 7 support, because it
  needs the compiler API.
- [`strict-type-checked`](https://typescript-eslint.io/users/configs/) is a superset of
  `recommended`, `recommended-type-checked` and `strict`. The project warns that it is "not
  covered by semver stability guarantees", so rules may change in minor versions. Exact pins make
  that a deliberate upgrade, not a surprise. `stylistic-type-checked` is additive. `all` is "not
  recommended" because its rules conflict.
- Type information comes from `parserOptions.projectService: true`. Use `disableTypeChecked` for
  any files outside a tsconfig.
- Rules you may want beyond strict, all type-aware, because the strict preset leaves them off:
  - `switch-exhaustiveness-check`
  - `strict-boolean-expressions`
  - `prefer-readonly` (not `prefer-readonly-parameter-types`, which is noise on DOM code
    [measured via Oxlint])
  - `no-unnecessary-qualifier`
  - `require-array-sort-compare`
  - `consistent-type-exports`
  - `promise-function-async`
  - `explicit-module-boundary-types`, a judgement call for a non-library
- ESLint core and `@eslint/js` 10.0.1 `recommended`, plus `eqeqeq`, `no-console` (allow
  `warn`/`error`), `prefer-const` and `no-param-reassign`.

### Biome

- `@biomejs/biome` 2.6.8 (2026-11-15). Type-aware rules live in the
  [`types` domain](https://biomejs.dev/linter/domains/): `noFloatingPromises`, `noMisusedPromises`,
  `useAwaitThenable`, `noUnnecessaryConditions`, `useExhaustiveSwitchCases`,
  `useStrictBooleanExpressions` and others. They run on **Biome's own inference engine, not `tsc`**,
  and are **nursery**: [`noFloatingPromises`](https://biomejs.dev/linter/rules/no-floating-promises/)
  "is experimental and the behavior can change at any time". Its Astro
  [support](https://biomejs.dev/internals/language-support/) for parsing, formatting and linting is
  "Experimental" behind `html.experimentalFullSupportEnabled`, and the `astro` domain's rules are
  all nursery.
- **Verdict:** Biome is the least strict of the three for this codebase. Its type-aware coverage is
  narrower and inferred, and its Astro support is experimental.

### Oxlint

- `oxlint` 1.94.0 (2026-11-16) and `oxlint-tsgolint` 7.0.2007 (2026-11-12). Type-aware linting has
  been [stable since 2026-07-22](https://oxc.rs/blog/2026-07-22-type-aware-linting-stable): 59 of
  typescript-eslint's 61 type-aware rules, with the same options, and 12 to 18 times faster than
  ESLint.
- `--type-check` reports `tsc` diagnostics from the same program, and the
  [docs](https://oxc.rs/docs/guide/usage/linter/type-aware.html) say it "can replace a separate
  `tsc --noEmit` step". It **requires TypeScript 7** (typescript-go).
- **Astro:** the [compatibility page](https://oxc.rs/compatibility) says "No template linting yet".
  In my run, Oxlint reported nothing from the `.astro` pages [measured].
- Built-in plugins: `eslint`, `typescript`, `unicorn` and `oxc` by default; `import`, `jsdoc`,
  `promise`, `node`, `vitest`, `jest` and others opt-in. There is **no native `regexp` or
  `playwright` plugin**. JS plugins with an ESLint-compatible API exist, but I did not verify that
  `eslint-plugin-playwright` loads through them.
- Using Oxlint with `--type-check` alongside `astro check` would mean two TypeScripts (7 for
  tsgolint, 6 for Astro), wired up with the alias above. It works, at the cost of two checkers that
  can disagree **[inferred]**.

### Plugins

- **Astro:** `eslint-plugin-astro` 3.2.1 (2026-09-17; peer `eslint >=10`; `typescript-eslint` an
  optional peer; Node `^22.22.3 || ^24.16 || >=26.3`, which the site's 22.23.4 meets). Use the
  `recommended` config. Its `all` config is "for testing, not for production use". Its
  [guide](https://ota-meshi.github.io/eslint-plugin-astro/user-guide/) documents type-aware linting
  in `.astro` through `parserOptions.project`. It does not mention `projectService`, so whether
  `projectService` works inside `.astro` is **[unverified]**. Optional extra:
  `jsx-a11y-strict` with `eslint-plugin-jsx-a11y`.
- **Playwright:** `eslint-plugin-playwright` 2.12.2 (2026-11-02). Use
  `playwright.configs['flat/recommended']` on `tests/**`. It includes `missing-playwright-await`,
  `no-wait-for-timeout`, `no-force-option` and `valid-expect-in-promise`. The README shows no `all`
  config. `@typescript-eslint/no-floating-promises` covers un-awaited promises in general.
- **High value:**
  - `eslint-plugin-unicorn` 80.0.0 (2026-11-15; peer `eslint >=10.4`; Node `>=22.13`): use
    `recommended`, and expect to turn off `prevent-abbreviations` and `filename-case` as taste.
  - `eslint-plugin-regexp` 3.3.3: use `recommended`. The code has around 28 regex literals, going by
    Oxlint's `require-unicode-regexp` hits.
- **Optional:**
  - `eslint-plugin-jsdoc` 65.4.11: `flat/recommended-typescript-flavor` for the plain JS files only.
  - `globals` 17.14.0: browser, worker and serviceworker globals for the JS blocks.
- **Not needed:** `eslint-config-prettier`. ESLint 10 core and typescript-eslint 8's presets no
  longer carry formatting rules, but check `stylistic-type-checked` for overlap **[unverified]**.

## 3. Formatting

| Tool | Version | Maturity | Astro | Notes |
|---|---|---|---|---|
| Prettier | 3.10.1 (2026-11-12) | Reference implementation | `prettier-plugin-astro` 1.1.2 (2026-10-23, peer `prettier ^3.5.3`), maintained by the Astro org; `@astrojs/language-server` peers on both | Slowest, but takes under a second at this size **[inferred]** |
| oxfmt | 0.79.0 (2026-11-16) | **Pre-1.0.** The changelog called it alpha at 0.12 and makes no stability statement since. It treats differences from Prettier as bugs. | Since 0.73.0 (2026-10-13) through `prettier-plugin-astro@1.x`, and "Format ts-in-astro as TSX" since 0.78.0 ([CHANGELOG](https://raw.githubusercontent.com/oxc-project/oxc/main/apps/oxfmt/CHANGELOG.md)). Peer `@astrojs/compiler-rs ^0.5.0` per the [compatibility page](https://oxc.rs/compatibility). | Very fast; Prettier-compatible output |
| Biome | 2.6.8 | Stable for JS/TS/CSS/JSON | Experimental (see above) | One binary for lint and format, but the linter is weaker |
| dprint | 0.65.0 | Stable | Through plugins; Astro support not checked **[unverified]** | Not considered further |

- **One tool for both lint and format without losing strictness:** none today. Biome loses
  type-aware depth and Astro maturity. Oxlint with oxfmt loses `.astro` linting and the
  regexp/Playwright rule sets, and couples you to TS 7 beside Astro's TS 6.
- **Recommendation:** Prettier with `prettier-plugin-astro`. Add `.prettierignore` entries for
  `dist/`, `.astro/`, `public/play/` and `prototype-console-site/`. Scope it to code, CSS and JSON
  rather than the repository Markdown the site renders. Migrating to oxfmt later is mechanical,
  because oxfmt targets Prettier's output and ships `--migrate prettier`.

## 4. Beyond lint and types

- **knip** 6.45.1 (2026-11-11): unused files, exports, dependencies and devDependencies. Its
  [Astro plugin](https://knip.dev/reference/plugins/astro) turns on when `astro` is a dependency.
  Its entries are `src/content.config.ts` and `src/pages/**/*.{astro,mdx,js,ts}`, and it reads
  `astro.config.*`. A Playwright plugin is not documented on that page, so knip may need explicit
  entries for `tests/**` and `playwright.config.ts` **[unverified]**. The repo has no `.mdx` and
  the content is Sätteri-rendered Markdown, so check the content-collection entries by hand.
  Recommended: it costs one devDependency, and it also catches unused devDependencies, which
  supports the repository's "recorded rationale per dependency" rule.
- **type-coverage** 2.31.2 (2026-11-04): `--at-least 100 --strict` counts `any` inside generics,
  non-null and `as` assertions, and `{}`/`Object` as uncovered; `--ignore-catch` exempts catch
  bindings. It uses the TS JS API (peer `typescript`), so it works with TS 6
  ([README](https://github.com/plantain-00/type-coverage)). With `strictTypeChecked`'s `no-unsafe-*`
  family plus `no-non-null-assertion` and `no-unnecessary-type-assertion`, most of what it measures
  is already a lint error. It adds value mainly as a ratchet on `as` assertions. Recommendation:
  optional; add it only if you want an explicit 100 % number **[inferred]**.
- **publint** 0.3.25: not relevant. It lints published package metadata, and `orcvs-site` is
  `private`.
- **ShellCheck** 0.11.0 (released 2025-08-04, latest;
  `gh api repos/koalaman/shellcheck/releases/latest`) and **shfmt** 3.14.1 (2026-09-06,
  `mvdan/sh`). Pin both in `mise.toml` through aqua (`aqua:koalaman/shellcheck`, `aqua:mvdan/sh`),
  matching the existing `actionlint`/`zizmor` entries. I did not run the registry lookup to confirm
  those aqua package names **[unverified]**. Strictest settings:
  - `shellcheck --severity=style --enable=all` (or a `.shellcheckrc` with `enable=all`), and
    `--external-sources` if the scripts `source` anything.
  - `shfmt -d -i 2 -ci -bn` (or read from `.editorconfig`).
  - Consider extending both to `scripts/*.sh`, which `check-tooling-contract.sh` may already cover
    in part.
  - actionlint already runs ShellCheck on workflow `run:` blocks when it is on `PATH`, so pinning
    ShellCheck in mise also tightens the existing actionlint gate **[inferred]**.
- **Dependency policy:**
  - `npm ci` already enforces the lockfile.
  - Add `npm audit --audit-level=low` (advisories) and `npm audit signatures` (registry signatures
    and provenance) to CI. Both need registry network access.
  - `lockfile-lint` 5.1.0 can assert https and the npm registry host for every resolved URL. That
    matters little with one registry; treat it as optional.
  - The existing `mise run audit_deps` is Cargo-only. The ticket's acceptance criterion that it
    passes is trivially met, but it does not audit npm. The npm audit needs its own task, or an
    addition to `audit_deps` **[inferred]**.
  - npm 11 adds release-age or minimum-age install policies. I did not verify which npm version
    has them, and Node 22.23.4 bundles npm 10 **[unverified]**.

## 5. Wiring

- **Pinning.** Exact versions with no `^`, matching the existing `site/package.json`. Commit
  `package-lock.json`, and install with `npm ci` everywhere. ShellCheck and shfmt come from mise
  with a pinned version. Because typescript-eslint's `strict` presets are outside semver, exact
  pins are required, not just preferred.
- **Tasks.** One `mise` task per gate, `dir = "site"`, following `site_build`'s style:
  - `site_fmt`: `prettier --check .`
  - `site_lint`: `eslint --max-warnings=0 .`
  - `site_types`: `astro check --minimumSeverity warning`, then `tsc -p ../synth/web` and
    `tsc -p ../console/assets`. `astro check` also takes `--noSync`; when run after
    `astro sync` it avoids regenerating `.astro/`
    **[unverified flag names; check `astro check --help`]**.
  - `site_shell`: `shellcheck` and `shfmt -d` on `site/scripts/**/*.sh`
  - `site_knip`: `knip`
  - An umbrella `site_check` that depends on all of them.
- **Tiers.**
  - Pull-request tier: run all of these in a separate `check` job in `site.yml`. It needs only
    `npm ci` and `astro sync`, not the console download or the Astro build, so it runs in parallel
    with `build` and finishes in well under a minute **[inferred]**. The strictest projects keep
    every deterministic, offline gate on the PR tier and move only slow or networked checks (full
    browser matrices, audits that can flake on advisory-DB updates) to merge or scheduled tiers.
  - Merge tier: `npm audit` / `npm audit signatures`.
  - `site.yml` has no `merge_group` trigger today, and its path filter does not include
    `synth/web/**` or `console/assets/sw.js`. The new job needs those paths added, or it will not
    run when only the plain JS changes.
- **Keeping it quick.**
  - Use `projectService` rather than listing several `project` tsconfigs.
  - Exclude `dist`, `public/play`, `.astro` and `prototype-console-site` from every tool. An
    `**/*` include that reaches `public/play/console.js` (a generated wasm-bindgen file) would slow
    every type-aware tool.
  - Cache `~/.npm` through the existing mise-action or setup cache.
  - ESLint's `--cache` is unnecessary in CI at this size.
- **Formatting commit.** As the ticket says, land `prettier --write` alone, then add its hash to
  `.git-blame-ignore-revs`.

## Proposed `site/tsconfig.json`

```jsonc
{
  "extends": "astro/tsconfigs/strictest",
  "compilerOptions": {
    "noPropertyAccessFromIndexSignature": true,
    "erasableSyntaxOnly": true,
    "noUncheckedSideEffectImports": true,
    "checkJs": true,
    "types": []
  },
  "include": [".astro/types.d.ts", "src/**/*", "tests/**/*", "*.ts", "*.mjs"],
  "exclude": ["dist", "public/play", "prototype-console-site"]
}
```

`types: []` matches the TS 7 default, and the measured run passed with it: Playwright and Astro
types come through imports.

Sketch for `synth/web/tsconfig.worklet.json`. `synth-host.js` gets a sibling config with the DOM
lib, and `console/assets/tsconfig.json` uses `lib: ["ES2024", "WebWorker"]`.

```jsonc
{
  "compilerOptions": {
    "target": "ES2024", "module": "ESNext", "moduleResolution": "Bundler",
    "allowJs": true, "checkJs": true, "noEmit": true, "skipLibCheck": true,
    "strict": true, "noUncheckedIndexedAccess": true, "exactOptionalPropertyTypes": true,
    "noImplicitReturns": true, "noUnusedLocals": true, "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true, "noPropertyAccessFromIndexSignature": true,
    "lib": ["ES2024"], "types": ["audioworklet"]
  },
  "files": ["synth-worklet.js"]
}
```

## Lint config sketch: `site/eslint.config.js`

```js
// @ts-check
import { defineConfig, globalIgnores } from "eslint/config";
import js from "@eslint/js";
import tseslint from "typescript-eslint";
import astro from "eslint-plugin-astro";
import playwright from "eslint-plugin-playwright";
import unicorn from "eslint-plugin-unicorn";
import regexp from "eslint-plugin-regexp";

export default defineConfig([
  globalIgnores(["dist/", ".astro/", "public/play/", "prototype-console-site/"]),
  js.configs.recommended,
  tseslint.configs.strictTypeChecked,
  tseslint.configs.stylisticTypeChecked,
  unicorn.configs.recommended,
  regexp.configs["flat/recommended"],
  astro.configs.recommended,
  {
    languageOptions: {
      parserOptions: { projectService: true, tsconfigRootDir: import.meta.dirname },
    },
    linterOptions: { reportUnusedDisableDirectives: "error", reportUnusedInlineConfigs: "error" },
    rules: {
      eqeqeq: "error",
      "@typescript-eslint/switch-exhaustiveness-check": "error",
      "@typescript-eslint/strict-boolean-expressions": "error",
      "@typescript-eslint/prefer-readonly": "error",
      "@typescript-eslint/require-array-sort-compare": "error",
      "@typescript-eslint/consistent-type-exports": "error",
      "@typescript-eslint/promise-function-async": "error",
    },
  },
  { files: ["tests/**"], extends: [playwright.configs["flat/recommended"]] },
  // `.astro` files: type-aware rules through the Astro parser. Check that
  // projectService resolves them, else point `project` at tsconfig.json.
  { files: ["**/*.astro"], languageOptions: { parserOptions: { parser: tseslint.parser } } },
]);
```

Check the config export names (`regexp.configs["flat/recommended"]`, `unicorn.configs.recommended`,
`reportUnusedInlineConfigs`) against each package's README when wiring it up. I took them from
memory and the READMEs above, and did not run them **[unverified]**.

devDependencies this adds, all exact:

- `typescript` 6.0.3
- `@astrojs/check` 0.9.12
- `@types/audioworklet` 0.0.102
- `eslint` 10.15.0
- `@eslint/js` 10.0.1
- `typescript-eslint` 8.73.0
- `eslint-plugin-astro` 3.2.1
- `eslint-plugin-playwright` 2.12.2
- `eslint-plugin-unicorn` 80.0.0
- `eslint-plugin-regexp` 3.3.3
- `prettier` 3.10.1
- `prettier-plugin-astro` 1.1.2
- `knip` 6.45.1
- optionally `@types/serviceworker` 0.0.210 and `type-coverage` 2.31.2

## Trade-offs and blockers

- **TS 7 is blocked** for `astro check` (peer `^5 || ^6`) and typescript-eslint (peer `<6.1.0`), and
  so is any tool on the TS JS API, including `type-coverage`. Everything stays on 6.0.3 until TS 7.1
  ships its API and these tools adopt it.
- **The Oxlint/oxfmt path is fast but less strict here today.** It does not lint Astro templates,
  has no regexp or Playwright rule sets natively, oxfmt is pre-1.0, and it needs TS 7 beside
  Astro's TS 6. It becomes attractive once those gaps close; the rule names and options already
  match typescript-eslint, so the switch would be cheap.
- **Biome:** its type-aware rules are nursery on a custom inference engine, and its Astro support is
  experimental.
- **The strict preset is outside semver.** Every typescript-eslint minor bump can add errors, so
  upgrades need deliberate review. Exact pins handle this.
- **Initial cost:** 58 type errors under the strictest tsconfig, mostly
  `noUncheckedIndexedAccess` in `sound-module.ts`, plus an expected lint backlog of similar size.
  The sw.js typing and the TS2801 false positive in `small.ts` (a deliberate guard TypeScript
  misreads) need code changes, not just config.
- **Path filter:** the site workflow does not watch `synth/web/` or `console/assets/`.

## Scope beyond the ticket: `scripts/*.ts`

The same TypeScript, ESLint and Prettier configs should later cover `scripts/*.ts` and
`scripts/tests/`. They run under Node's type stripping, which `erasableSyntaxOnly` enforces at
type-check time. Those files would need their own tsconfig with
`module`/`moduleResolution: "nodenext"`, `allowImportingTsExtensions`, `types: ["node"]`, and
`@types/node` pinned to Node 22, plus `eslint-plugin-n` 18.5.0 if wanted. The tooling lives in
`site/package.json` today, so covering `scripts/` means either a root `package.json` for the tools
or running them from `site/` with an explicit path. Decide that in the follow-up the ticket names
**[inferred]**.

## Could not verify

- That `astro check` passes, or what it reports, with Starlight 0.42.7 and Sätteri. I did not run
  it, because it would have needed packages installed in the repo. The 58-error count comes from
  plain `tsc` over `.ts`/`.mjs` and excludes `.astro` diagnostics.
- That `astro check` works, or fails cleanly, against TS 7 beyond the language-server release note.
- Whether `projectService` handles `.astro` files in `eslint-plugin-astro` 3.2.1.
- The exact names of the two typescript-eslint type-aware rules tsgolint lacks.
- Whether `eslint-plugin-playwright` loads as an Oxlint JS plugin.
- The aqua package names for ShellCheck and shfmt in mise's registry, and ShellCheck 0.11.0's
  results. I ran 0.9.0.
- The TS 7.1 release date. Secondary sources disagree, and Microsoft gives none.
- The flat-config export names in the ESLint sketch, and `astro check`'s `--noSync` and
  `--minimumSeverity` flags.
- npm's minimum-release-age policy and which npm version ships it.
- That typescript-eslint's `dot-notation` honours `noPropertyAccessFromIndexSignature`, which I
  recalled and did not re-read.
