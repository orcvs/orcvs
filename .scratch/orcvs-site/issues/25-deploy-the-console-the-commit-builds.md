# 25 — Deploy the console the commit builds

**What to build:** The production site build ships the console built from its own commit, or
fails. Today `site/scripts/console.sh` copies whatever console is on the `console-web` release when
Cloudflare's build starts, which races `.github/workflows/console-web.yml` publishing the new one:
#323 shipped the 808 beat at 22:25 on 2026-10-28, the console was published at 22:26:54, and
orcvs.com kept the previous console. The deploy-hook step that was meant to close the race needs a
`CLOUDFLARE_DEPLOY_HOOK` secret that was never set, and skips silently without it. See
[ADR 0092](../../../docs/adr/0092-the-site-deploys-only-the-console-its-commit-builds.md).

**Blocked by:** none

**Status:** resolved

- [x] `scripts/console-key.sh` prints one key per console: the same key for two commits that differ
      only outside `console-web.yml`'s path filter, and a different key when a path in it changes.
- [x] On `main`, `console-web.yml` publishes each console as `console-web-<key>.tar.gz` on the
      `console-web` release, keeps the rolling `console-web.tar.gz`, and keeps only the newest 20
      keyed archives.
- [x] When `WORKERS_CI_BRANCH` is `main`, `console.sh` waits for its commit's keyed console,
      retrying every 20 seconds for up to 15 minutes, and installs it into `site/public/play/`.
- [x] When the keyed console never arrives, the build fails with a message naming the key, and
      nothing is deployed.
- [x] Any other build (a preview, `site.yml`, a local build) behaves as before.
- [x] The deploy-hook step and every mention of `CLOUDFLARE_DEPLOY_HOOK` are gone.
- [x] A test of `console.sh` covers each case above without the network.

## Comments

### Built, 2026-10-28

`scripts/console-key.sh` hashes `git ls-tree HEAD` over the paths it reads from
`console-web.yml`'s filter, so the filter is the one list of what the console is built from. The
filter gained `.cargo/config.toml`, whose rustflags the WebAssembly build reads, and the key script
itself. The `publish` job uploads `console-web-<key>.tar.gz` with no queue, since each key names one
archive; a queued `latest` job replaces the rolling archive and prunes keyed ones beyond 20.
`site/scripts/tests/console.sh` stubs `curl` and `sleep` on `PATH` and runs both scripts in a
throwaway repository; `site.yml` runs it.
