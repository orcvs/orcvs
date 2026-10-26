# 05 — Deploy to Cloudflare with a preview per pull request

**What to build:** The composed site deploys to Cloudflare as a Worker with static assets and no
script. `site/wrangler.jsonc` names the build output as `assets.directory`, with
`not_found_handling` set to `404-page` and preview URLs on. A GitHub Actions workflow runs the
ticket 03 build once, keeps it as an artifact, and deploys that artifact on pushes to `main` and
as a preview, with its URL posted, on each pull request that changes the site. A `_headers` file sets caching so a deploy replaces
`/play/`'s HTML and JS. `/play/` keeps its service worker, scoped to `/play/`, and a deploy must
replace a cached console: this ticket picks file hashing (off today in `console/Trunk.toml`) or a
versioned worker. See `../spec.md` (Hosting, Service worker).

**Blocked by:** 03

**Status:** ready-for-human

Toby provides the Cloudflare account, an API token stored as a repository secret, and the domain.
The workflow is written so it does nothing until those exist.

- [x] A test or recorded check shows a second deploy replaces the console a returning reader had
      cached.
- [ ] A pull request that changes the site gets a preview URL.
- [x] The workflow passes `bash scripts/check-tooling-contract.sh`, `actionlint` and
      `zizmor --offline .github/workflows`, and its token has the least scope the deploy needs.

## Comments

### Built, 2026-10-26; waiting on the Cloudflare account

`.github/workflows/site.yml` builds once with `site/build.sh`, around the `console-web` artifact
the console's own workflow builds, and uploads `site/dist/` as the `site-<sha>` artifact. The `test` job runs the Playwright suite against that artifact; `deploy`
(pushes to `main`) and `preview` (pull requests from this repository) download the same artifact
and run Wrangler 4.149.0 through `npx`. So what reaches Cloudflare is byte for byte what the tests
passed, and a release can publish the same build (ticket 12). On 2026-10-26 Toby asked for this
over Cloudflare's GitHub integration, which builds from source in its own image and cannot deploy
a build made elsewhere. The Cloudflare steps skip until the
`CLOUDFLARE_API_TOKEN` secret and `CLOUDFLARE_ACCOUNT_ID` variable exist. The preview comment is
one comment per pull request, edited on each push.

The console keeps fixed file names and its existing worker: navigations, `console.js` and
`console_bg.wasm` are network-first, and `_headers` makes every `/play/` file revalidate. So a
deploy needs neither hashing nor a new cache name, and the ticket's choice between them is answered
with "neither": either would repeat what the worker and `_headers` already guarantee.
`site/tests/deploy.spec.ts` proves it by replacing the served page and script, with the page
naming the new script's integrity hash as a build would, under a reader whose worker controls the
page. The new console must start, and the test fails if the worker answers navigations from its
cache.

For Toby: create an account API token with only Account → Workers Scripts → Edit, store it as the
`CLOUDFLARE_API_TOKEN` repository secret, set the `CLOUDFLARE_ACCOUNT_ID` repository variable, and
name the domain. The first pull request after that gets a preview comment, which closes the
remaining box.
