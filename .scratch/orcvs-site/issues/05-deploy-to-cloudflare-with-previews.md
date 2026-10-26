# 05 — Deploy to Cloudflare with a preview per pull request

**What to build:** The composed site deploys to Cloudflare as a Worker with static assets and no
script. `site/wrangler.jsonc` names the build output as `assets.directory`, with
`not_found_handling` set to `404-page` and preview URLs on. Cloudflare's GitHub app builds the site
with `npm run build` in `site/` and deploys it on pushes to `main`, and as a preview on each pull request that
changes the site. A `_headers` file sets caching so a deploy replaces
`/play/`'s HTML and JS. `/play/` keeps its service worker, scoped to `/play/`, and a deploy must
replace a cached console: this ticket picks file hashing (off today in `console/Trunk.toml`) or a
versioned worker. See `../spec.md` (Hosting, Service worker).

**Blocked by:** 03

**Status:** ready-for-human

Toby provides the Cloudflare account, connects Cloudflare's GitHub app, and names the domain.

- [x] A test or recorded check shows a second deploy replaces the console a returning reader had
      cached.
- [ ] A pull request that changes the site gets a preview URL.
- [x] The workflow passes `bash scripts/check-tooling-contract.sh`, `actionlint` and
      `zizmor --offline .github/workflows`, and GitHub holds no Cloudflare API token.

## Comments

### Built, 2026-10-26; waiting on the Cloudflare account

Cloudflare builds and deploys the site through its GitHub app, as Toby asked on 2026-10-26, so
GitHub holds no Cloudflare token. Its build runs `npm run build`, whose `prebuild` step (`site/scripts/console.sh`) downloads the console
`.github/workflows/console-web.yml` publishes from `main` on the `console-web` release and builds
only the Astro site. That workflow posts to a deploy hook after publishing, so a console change
reaches the live site without a site change. `.github/workflows/site.yml` runs the same script and
the Playwright suite, and deploys nothing.

The console keeps fixed file names and its existing worker: navigations, `console.js` and
`console_bg.wasm` are network-first, and `_headers` makes every `/play/` file revalidate. So a
deploy needs neither hashing nor a new cache name, and the ticket's choice between them is answered
with "neither": either would repeat what the worker and `_headers` already guarantee.
`site/tests/deploy.spec.ts` proves it by replacing the served page and script, with the page
naming the new script's integrity hash as a build would, under a reader whose worker controls the
page. The new console must start, and the test fails if the worker answers navigations from its
cache.

For Toby, in the Cloudflare dashboard: Workers & Pages → Create → Import a repository, choose
`orcvs/orcvs` (this installs Cloudflare's GitHub app), and set:

- Worker name: `orcvs-site`, the `name` in `site/wrangler.jsonc`.
- Root directory: `site`. Build command: `npm run build`, Cloudflare's default.
- Deploy command: `npx wrangler@4.149.0 deploy`. Turn on builds for non-production branches, with
  `npx wrangler@4.149.0 preview` as the preview command.
- Build watch paths: `site/*`, `docs/first-result.md`, `docs/functions.md` and `mise.toml`, so a
  console-only push does not rebuild the site before its console is published.
- Under Builds → Deploy Hooks, a hook for `main`; store its URL as the `CLOUDFLARE_DEPLOY_HOOK`
  repository secret. It can only start a build.

Then name the domain. A preview needs a published console, so previews work once this pull request
has merged and `main` has published one.
