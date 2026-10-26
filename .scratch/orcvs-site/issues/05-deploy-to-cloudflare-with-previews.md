# 05 — Deploy to Cloudflare with a preview per pull request

**What to build:** The composed site deploys to Cloudflare as a Worker with static assets and no
script. `site/wrangler.jsonc` names the build output as `assets.directory`, with
`not_found_handling` set to `404-page` and preview URLs on. A GitHub Actions workflow runs the
ticket 03 build, then `wrangler deploy` on pushes to `main`, and uploads a preview for each pull
request that changes the site, posting its URL. A `_headers` file sets caching so a deploy replaces
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

`site/wrangler.jsonc` and `site/public/_headers` configure the Worker, and the deploy steps sit at
the end of the `site` job in `.github/workflows/site.yml`, so they use the build the tests just
passed. A preview's URL goes to the job summary and to one comment on the pull request, edited on
each upload. They skip until the repository has the `CLOUDFLARE_API_TOKEN` secret and the
`CLOUDFLARE_ACCOUNT_ID` variable. `wrangler deploy --dry-run` reads the 53 built files with no
bindings.

The console keeps fixed file names and its existing worker: navigations, `console.js` and
`console_bg.wasm` are network-first, and `_headers` makes every `/play/` file revalidate. So a
deploy needs neither hashing nor a new cache name, and the ticket's choice between them is answered
with "neither": either would repeat what the worker and `_headers` already guarantee. `site/tests/deploy.spec.ts` proves it by
replacing the served page and script, with the page naming the new script's integrity hash as a
build would, under a reader whose worker controls the page. The new console must start, and the
test fails if the worker answers navigations from its cache.

Wrangler 4.149.0 runs through `npx` in the workflow rather than from `site/package.json`: as a
dependency it adds about 180 MB to every site install, and its local-dev runtime carries an
undici advisory that the deploy never runs.

For Toby: create an account API token with only Account → Workers Scripts → Edit, store it as the
`CLOUDFLARE_API_TOKEN` repository secret, set the `CLOUDFLARE_ACCOUNT_ID` repository variable, and
name the domain so it can go in `routes`. The first pull request after that shows its preview URL
in the job summary, which closes the remaining box.
