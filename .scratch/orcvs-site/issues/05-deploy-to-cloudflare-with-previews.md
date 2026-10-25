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

**Status:** ready-for-agent

Toby provides the Cloudflare account, an API token stored as a repository secret, and the domain.
The workflow is written so it does nothing until those exist.

- [ ] A test or recorded check shows a second deploy replaces the console a returning reader had
      cached.
- [ ] A pull request that changes the site gets a preview URL.
- [ ] The workflow passes `bash scripts/check-tooling-contract.sh`, `actionlint` and
      `zizmor --offline .github/workflows`, and its token has the least scope the deploy needs.
