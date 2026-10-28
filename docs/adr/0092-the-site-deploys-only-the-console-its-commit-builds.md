# The site deploys only the console its commit builds

Status: accepted. Amends the deploy hook in `.scratch/orcvs-site/spec.md` (Build) and ticket 05. Settled in `.scratch/orcvs-site/issues/25-deploy-the-console-the-commit-builds.md`.

A merge to `main` starts two builds at once. Cloudflare builds the site, and its `prebuild` step copies whatever console is on the `console-web` release at that moment. GitHub Actions builds the console from the same commit and publishes it about a minute and a half later. When Cloudflare copies first, orcvs.com ships the previous console and nothing reports it: #323 merged the 808 beat at 22:25 on 2026-10-28, the new console was published at 22:26:54, and the live site kept the old one. The step meant to close the gap posted to a Cloudflare deploy hook stored as a repository secret that was never set, and skipped quietly without it.

## Decision

**A commit's console has a key.** The key is one hash over the git objects of every path the console is built from, which is exactly the path filter of `.github/workflows/console-web.yml`. `scripts/console-key.sh` prints it by reading that filter, so the two cannot drift. Two commits share a key exactly when they build the same console.

**The console is published under its key.** On `main`, `console-web.yml` uploads each build as `console-web-<key>.tar.gz` on the `console-web` release, beside the rolling `console-web.tar.gz` that local builds and previews take.

**The production site build waits for its key.** When Cloudflare builds `main` (`WORKERS_CI_BRANCH` is `main`), `site/scripts/console.sh` computes the key of the commit it builds and downloads `console-web-<key>.tar.gz`, retrying every 20 seconds for up to 15 minutes. If the console never arrives, the build fails and names the key, so Cloudflare deploys nothing and reports the failure on the commit. The site never deploys a console other than the one its commit builds.

**No deploy hook.** The site needs no secret and GitHub holds no Cloudflare credential of any kind. A site build that fails because the console build failed or ran late is retried from Cloudflare's dashboard.

## Considered options

**The deploy hook.** It fixes the race only while a secret nobody is reminded of stays set, and a missing secret fails silently. The race is between two builds the repository already controls, so the repository can close it without a credential.

**Wait for the published console to name the commit.** A commit that does not change the console still needs the newest console that an earlier commit built, so the site would have to find that commit in history, which a shallow clone may not have. The key needs only the commit's own tree.

**Build the console inside the site build.** Toby decided on 2026-10-26 that the site build never compiles the console.

## Consequences

- Cloudflare's build of the site must run on pushes that change the console, so its build watch paths must include the console's paths or be left unset.
- A production site build can wait up to 15 minutes of Cloudflare's 20-minute build limit.
- Keyed archives accumulate on the release. The publish workflow keeps the newest 20 and deletes the rest.
- A pull request's preview and a local build still take the rolling console, because a pull request's console is never published.
