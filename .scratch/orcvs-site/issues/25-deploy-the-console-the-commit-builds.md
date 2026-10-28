# 25 — Deploy the console the commit builds

**What to build:** The production site build ships the console built from its own commit, or
fails. Today `site/scripts/console.sh` copies whatever console is on the `console-web` release when
Cloudflare's build starts, which races `.github/workflows/console-web.yml` publishing the new one:
#323 shipped the 808 beat at 22:25 on 2026-10-28, the console was published at 22:26:54, and
orcvs.com kept the previous console. The deploy-hook step that was meant to close the race needs a
`CLOUDFLARE_DEPLOY_HOOK` secret that was never set, and skips silently without it. See
[ADR 0092](../../../docs/adr/0092-the-site-deploys-only-the-console-its-commit-builds.md).

**Blocked by:** none

**Status:** ready-for-agent

- [ ] `scripts/console-key.sh` prints one key per console: the same key for two commits that differ
      only outside `console-web.yml`'s path filter, and a different key when a path in it changes.
- [ ] On `main`, `console-web.yml` publishes each console as `console-web-<key>.tar.gz` on the
      `console-web` release, keeps the rolling `console-web.tar.gz`, and keeps only the newest 20
      keyed archives.
- [ ] When `WORKERS_CI_BRANCH` is `main`, `console.sh` waits for its commit's keyed console,
      retrying every 20 seconds for up to 15 minutes, and installs it into `site/public/play/`.
- [ ] When the keyed console never arrives, the build fails with a message naming the key, and
      nothing is deployed.
- [ ] Any other build (a preview, `site.yml`, a local build) behaves as before.
- [ ] The deploy-hook step and every mention of `CLOUDFLARE_DEPLOY_HOOK` are gone.
- [ ] A test of `console.sh` covers each case above without the network.
