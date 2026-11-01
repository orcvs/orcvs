# Every site build takes the console its commit builds

Status: accepted. Amends [ADR 0093](0093-the-site-deploys-only-the-console-its-commit-builds.md).
Settled in `.scratch/orcvs-site/issues/29-preview-the-console-a-branch-builds.md`.

A pull request's preview runs `main`'s console, because ADR 0093 publishes only consoles built on
`main` and every build other than production takes the rolling `console-web.tar.gz`. A console
change, such as the 303 voice in #340, cannot be heard on its preview until it merges.

## Decision

**A push that changes the console publishes it under its key.** `console-web.yml` keeps ADR 0093's
path filter and runs on a push to any branch except `gh-pages` and the merge queue's, so it runs
only when the push changes a path the console is built from (Toby, 2026-11-01). Its first job
computes the commit's console key and lists the `console-web` release, reading every page and
failing on any API error other than a missing release. When the key is already published the run
builds nothing, because one key is one console. Otherwise it builds the console and publishes it.

**A branch that changes nothing in the console takes `main`'s.** Its key is the key of the `main`
commit it branched from, which `main` published when it last changed the console.

**Branches publish under their own name.** `main` publishes `console-web-<key>.tar.gz`, as before.
Any other branch publishes `console-preview-<key>.tar.gz`. `main` counts only its own archives as
published, and production's site build takes only `main`'s, so a console a branch published never
reaches orcvs.com. A branch counts either, since `main`'s archive of its key is the same console.

**An archive is never replaced.** `publish` lists the release again and leaves an archive another
run already published, and uploads without `--clobber`. Builds of one archive run one at a time.

**No push cancels another's run.** A cancelled run publishes nothing and leaves its commit's
preview waiting for a console that never comes, so every pushed commit's run finishes.

**Every Cloudflare build waits for its key.** `site/scripts/console.sh` applies ADR 0093's wait to
any build where `WORKERS_CI_BRANCH` is set. Production waits for `main`'s archive; a preview takes
either. Dependabot's pushes get a read-only token and cannot publish, so a `dependabot/` branch's
build takes the rolling console, as a local build does.

**The key still reads the path filter.** The console script's test fails when a workspace member,
a file a crate embeds from outside itself, or a script a Trunk hook runs is missing from it.

**The release keeps the newest 100 of `main`'s archives and 50 of the branches'**, and never prunes
`main`'s current key. A branch so far behind `main` that its key was pruned rebuilds it by running
Console web on the branch, which the failed preview's message names.

## Considered options

**A label that opts a pull request in.** It leaves every unlabelled preview running `main`'s
console without saying so, and someone has to remember the label on each console change.

**A manual `workflow_dispatch` run on the branch.** The same silent default, and the preview build
that started with the push has already taken the rolling console by the time anyone runs it.

**Publish from the `pull_request` trigger.** Cloudflare builds a branch on push, before a pull
request exists, and the `pull_request` run builds the merge with `main` rather than the head
Cloudflare builds.

**Run on every push and skip by key.** It needs no path filter and never meets a pruned key, but
every push of every branch runs a job for a console it did not change.

## Consequences

- A push that changes the console builds it, as a pull request into `main` already did; stacked
  branches now build too. Any other push runs nothing.
- A preview of a console change waits for its console build, a few minutes warm.
- A push superseded by a newer one on the same branch still builds, so a branch pushed several
  times quickly builds several consoles.
- A Dependabot branch's preview runs `main`'s console. Its console is still built, so a break shows
  on its run.
- A branch's run holds `contents: write`, as it must to publish. A pushed branch already runs its
  own version of every workflow, so this grants nothing a push could not already take; the
  separate names keep an honest branch run from ever touching `main`'s archives.
- Fork pull requests no longer run the console build, since forks do not push to this repository.
