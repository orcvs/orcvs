# Every site build takes the console its commit builds

Status: accepted. Amends [ADR 0093](0093-the-site-deploys-only-the-console-its-commit-builds.md).
Settled in `.scratch/orcvs-site/issues/29-preview-the-console-a-branch-builds.md`.

A pull request's preview runs `main`'s console, because ADR 0093 publishes only consoles built on
`main` and every build other than production takes the rolling `console-web.tar.gz`. A console
change, such as the 303 voice in #340, cannot be heard on its preview until it merges.

## Decision

**Every pushed commit's console is published under its key.** `console-web.yml` runs on a push to
any branch except `gh-pages` and the merge queue's. Its first job computes the commit's console key
and looks for `console-web-<key>.tar.gz` on the `console-web` release. When the key is already
published, by `main` or by any branch, the run builds nothing, because one key is one console.
Otherwise it builds the console and publishes it under its key.

**Every Cloudflare build waits for its key.** `site/scripts/console.sh` applies ADR 0093's wait to
any build where `WORKERS_CI_BRANCH` is set, preview or production. A local build still takes the
rolling console.

**The key's paths live in `scripts/console-key.sh`.** The workflow has no path filter left for the
key to read, and needs none: it runs for every commit and checks the key itself. The list is the
same set of paths as ADR 0093's filter, so keys published before this change stay valid.

**`main` takes a console a branch already built.** A pull request merged with an up-to-date branch
has the same key as its head, so `main`'s run finds the console published and only points the
rolling console at it. The key covers every input of the build, including the workflow and the tool
pins, so the console is the one `main` would build.

**The release keeps the newest 100 keyed archives.** A pruned key is rebuilt by the next commit that
needs it, because every run checks for its key before trusting it.

## Considered options

**A label that opts a pull request in.** It leaves every unlabelled preview running `main`'s
console without saying so, and someone has to remember the label on each console change.

**A manual `workflow_dispatch` run on the branch.** The same silent default, and the preview build
that started with the push has already taken the rolling console by the time anyone runs it.

**Publish from the `pull_request` trigger.** Cloudflare builds a branch on push, before a pull
request exists, and the `pull_request` run builds the merge with `main` rather than the head
Cloudflare builds. A path filter also skips branches that change nothing in the console, whose key
may have been pruned.

## Consequences

- Every push runs a job of a few seconds. A push that changes the console builds it, as a pull
  request into `main` already did; stacked branches now build too.
- A preview of a console change waits for its console build, a few minutes warm.
- A push superseded by a newer one on the same branch is cancelled, so its preview fails after 15
  minutes. The newer push's preview is the one that matters.
- Dependabot's pushes get a read-only token, so their consoles are built but not published and
  their previews fail. Rebasing the branch by hand publishes it.
- Anyone who can push a branch can publish a keyed console. Pushing already needs write access to
  the repository, which can edit the release directly.
- Fork pull requests no longer run the console build, since forks do not push to this repository.
