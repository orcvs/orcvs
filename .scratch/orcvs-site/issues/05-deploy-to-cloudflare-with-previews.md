# 05 — Deploy to Cloudflare with a preview per pull request

**What to build:** The composed site deploys to Cloudflare as a Worker with static assets and no
script. `site/wrangler.jsonc` names the build output as `assets.directory`, with
`not_found_handling` set to `404-page` and preview URLs on. Workers Builds, Cloudflare's GitHub
integration, runs the ticket 03 build, deploys pushes to `main`, and posts a preview URL on each
pull request that changes the site. A `_headers` file sets caching so a deploy replaces
`/play/`'s HTML and JS. `/play/` keeps its service worker, scoped to `/play/`, and a deploy must
replace a cached console: this ticket picks file hashing (off today in `console/Trunk.toml`) or a
versioned worker. See `../spec.md` (Hosting, Service worker).

**Blocked by:** 03

**Status:** ready-for-human

Toby connects the repository in the Cloudflare dashboard, with the settings below, and supplies
the domain.

- [x] A test or recorded check shows a second deploy replaces the console a returning reader had
      cached.
- [ ] A pull request that changes the site gets a preview URL.
- [x] The build installs only pinned, checksummed tools, and the repository holds no Cloudflare
      credential.

## Comments

### Built, 2026-10-26; waiting on the dashboard

On 2026-10-26 Toby chose Cloudflare's GitHub integration over a deploy from GitHub Actions, so
the repository holds no API token and Cloudflare posts the preview URLs itself. Its build image
has Node but no Rust, so `site/cloudflare-build.sh` installs rustup 1.28.2 and Trunk from
mise.toml's pin, each checked against a SHA-256, then runs `site/build.sh`, the same build
`mise run site_build` and the `site` workflow run. The script ran end to end in this session from
an empty home directory. `site/.node-version` picks the image's Node; the script fails if it
drifts from `mise.toml`.

The console keeps fixed file names and its existing worker: navigations, `console.js` and
`console_bg.wasm` are network-first, and `_headers` makes every `/play/` file revalidate. So a
deploy needs neither hashing nor a new cache name, and the ticket's choice between them is answered
with "neither": either would repeat what the worker and `_headers` already guarantee.
`site/tests/deploy.spec.ts` proves it by replacing the served page and script, with the page
naming the new script's integrity hash as a build would, under a reader whose worker controls the
page. The new console must start, and the test fails if the worker answers navigations from its
cache.

For Toby, in the Cloudflare dashboard under Workers & Pages, create a Worker by importing
`orcvs/orcvs` from GitHub, then set:

- Worker name: `orcvs-site`, matching `site/wrangler.jsonc`.
- Root directory: `site`.
- Build command: `bash cloudflare-build.sh`.
- Deploy command: `npx wrangler@4.149.0 deploy`.
- Preview builds: on, with preview command `npx wrangler@4.149.0 preview`.
- Production branch: `main`.
- Build watch paths: `site/*`, `console/*`, `orcvs/*`, `lang/*`, `Cargo.toml`, `Cargo.lock`,
  `rust-toolchain.toml`, `mise.toml`, `docs/first-result.md`, `docs/functions.md`.
- The domain, once chosen, under the Worker's Domains & Routes.

The first pull request after that shows its preview URL in a Cloudflare comment, which closes the
remaining box.
