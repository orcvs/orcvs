# 29 — Preview the console a branch builds

**What to build:** A branch's Cloudflare preview runs the console that branch builds. Today only
`main` publishes keyed consoles (ADR 0093), so every preview takes `main`'s rolling console, and a
console change such as the 303 voice (#340) cannot be heard before it merges. See
[ADR 0101](../../../docs/adr/0101-every-site-build-takes-the-console-its-commit-builds.md).

**Blocked by:** 25

**Status:** resolved

- [x] `console-web.yml` runs on pushes to every branch except `gh-pages` and the merge queue's.
- [x] A run whose console key is already on the `console-web` release builds nothing; any other
      run builds the console and publishes it under its key, except for Dependabot's read-only pushes.
- [x] On `main` the rolling console is pointed at `main`'s key whether or not the run built it, and
      the release keeps the newest 100 keyed archives.
- [x] `scripts/console-key.sh` lists the console's paths itself and keys the same paths as before.
- [x] Every Cloudflare build (`WORKERS_CI_BRANCH` set) waits for its commit's keyed console; a local
      build takes the rolling console.
- [x] `site.yml` builds and tests the pull request's head with that same console.
- [x] The test of `console.sh` covers a preview waiting for its console, a preview whose console is
      already published, and a local build.
