# 02 — Refresh the tooling contract's stale action pins

**What to build:** `scripts/check-tooling-contract.sh` runs to completion against the workflow as dependabot has left it, so the gates that follow it can be trusted to have run at all.

**Blocked by:** None — can start immediately.

**Status:** in-review

- [x] The contract passes against the current workflow.
- [x] Both stale pins match the workflow, not just the first one.
- [x] The decision on whether these assertions name an exact version or only require a commit-sha pin with a version comment is recorded in this issue.

## Comments

Two pins are stale, not one. The script exits at its first failure, so the checkout mismatch hid the mise-action mismatch behind it. A contract that reports one problem at a time understates how far it has drifted.

Dependabot bumped both actions on 31 August in one grouped pull request. The contract has been failing since, and nothing ran it — which is issue 03.

The version question is worth settling rather than defaulting. Exact pins mean every future action bump needs a one-line edit here, which is a forced review of the bump once 03 makes the contract fail loudly, and matches the repository's stance that dependency changes carry a recorded rationale. Asserting only "pinned to a commit sha with a version comment" keeps the security property and removes the edit. The choice matters less than recording which one was made and why.

## Decision — the assertions name an exact version

`actions/checkout` is pinned as `# v7.0.1` and `jdx/mise-action` as `# v4.3.0`, and the contract
asserts those exact strings rather than "a commit sha with any version comment".

The looser shape assertion keeps the security property — a mutable tag can never be what CI resolves
— and removes a one-line edit from every dependabot bump. The exact version keeps something the
shape cannot: `AGENTS.md` requires a recorded rationale for a dependency change, and issue 03 makes
the contract fail loudly on a pull request. Together those turn each action bump into a change
somebody has to look at and describe, which is the same treatment a cargo bump already gets. The
edit is the point rather than the cost.

The failure this pair of pins produced argues the same way. Both went stale on 31 August in one
grouped pull request and stayed stale because nothing ran the contract. Under the shape assertion
that drift would have been invisible rather than merely unreported, because there would have been
nothing to drift from.

### What that costs, concretely

Once issue 03 wires the contract into `check_pull_request`, the next `actions/checkout` or
`jdx/mise-action` patch bump fails the Dependabot pull request itself, and merging it means editing
the version string in this script by hand. `Swatinem/rust-cache` is asserted as `# v2` and breaks the
same way on a major bump. That is the forced review working rather than a defect — but it is worth
knowing before the first bump lands, because the failure will look like a broken gate rather than a
prompt. If the friction proves worse than the review is worth, the replacement is a version-agnostic
tail such as `# v[0-9]+([.][0-9]+){0,2}$`, which keeps the sha-pinning property and drops the prompt.
