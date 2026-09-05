# 13 — Bump the mise tool pins automatically

**What to build:** Every version this repository pins is bumped by something that watches the file it lives in. `mise.toml`'s `[tools]` table is watched by nothing.

**Blocked by:** None — can start immediately.

**Status:** ready-for-human

- [ ] Something automatic proposes a bump when a pinned tool in `mise.toml` releases.
- [ ] The chosen mechanism does not duplicate or displace the Dependabot config that already owns `Cargo.toml`, the actions, and the Rust channel.
- [ ] `docs/tooling.md` says what watches `mise.toml`.

## Comments

`.github/dependabot.yml` now covers three ecosystems: `cargo` for the workspace manifests, `github-actions` for the workflow pins, and `rust-toolchain` for the channel in `rust-toolchain.toml`. That third entry closed half of this gap — the toolchain had sat at 1.98.0 while 1.98.1 was current, precisely because nothing read the file.

The other half stays open. `mise.toml` pins seven tools — `cargo-nextest`, `cargo-deny`, `trunk`, `wasm-pack`, `node`, `actionlint`, and `zizmor` — and every one of them moves only when a human edits the line. `docs/tooling.md` closes with "upgrade each version deliberately in its source-of-truth file", which describes the intent accurately and provides no prompt. Nothing tells anyone a release happened.

What it costs is not urgency but silence. A stale `cargo-deny` is the one that bites: the advisory database it reads is fetched fresh, but the tool that reads it ages, and a version old enough to miss an advisory format change fails open rather than loudly. The rest is slower: a stale `trunk` or `wasm-pack` diverges from the browser toolchain the application actually ships against, and the divergence is discovered by a build breaking rather than by a bump landing.

Dependabot cannot close it. There is no `mise` value for `package-ecosystem`; the request is dependabot-core #12320, open since May 2025 with the project status "On Hold", and a community prototype (#14500) was declined in April 2026 on governance rather than technical grounds — GitHub asks an upstream maintainer to commit to long-term support of a new ecosystem, and the mise project was not in a position to. Treat mise support as not arriving on any horizon this repository should plan around.

Three options, none free:

Renovate has a first-class `mise` manager and would cover every backend this repository uses: the `cargo:` pins map to its crate datasource and `node` to its node-version datasource. It also has a `rust-toolchain` manager. The cost is that Renovate is a whole second bot: either it runs beside Dependabot and two bots open pull requests against `Cargo.toml`, or the existing three-ecosystem Dependabot config is migrated wholesale. Adopting a dependency bot is a maintainer's decision about the repository, not a change to a checkout.

A scheduled workflow running `mise up --bump` and opening a pull request is the smallest thing that would work. `mise outdated --bump` reports and `mise up --bump` rewrites the pins in place, preserving the precision of each. It reuses the shape `.github/workflows/advisories.yml` already establishes — a weekly cron, mise-action, one task — and needs a pull-request-opening action, which is a `contents: write` grant this repository presently gives to exactly one job.

Doing nothing and writing the review cadence down is defensible on a repository this size, provided it is written down rather than assumed. That is what this file is for until one of the other two is chosen.

The decision is a maintainer's: it is about which bots run against the repository and what write permission they hold, not about what a checkout contains.
