# 06 — Run Miri deliberately against the Source model

**What to build:** Miri can be run against the tests that reach the workspace's one `unsafe` block,
by a command that exists and a job someone can trigger, without becoming a check every pull request
pays for.

**Blocked by:** None — can start immediately.

**Status:** ready-for-human — the task, the workflow, the contract rules and the documentation are
written and their static gates pass, but Miri itself has never been run: the interpreter has not
executed a single one of the selected tests, here or in CI. The first real execution belongs to the
manual dispatch this issue exists to create.

- [x] A task runs Miri over the Source model tests, installing the nightly toolchain it needs rather
      than changing the pinned channel. `mise.toml` `[tasks.miri]` installs `nightly` with the `miri`
      component and runs `cargo +nightly miri nextest run`; `rust-toolchain.toml` is untouched.
      **Written, never executed** — see the Status above.
- [x] The run is scoped by test filter, not by crate. `-E 'test(/^source::model::test::/)'` selects
      68 tests, confirmed by `cargo nextest list`, all in the `orcvs` lib unit-test binary and all in
      `orcvs/src/source/model.rs`, the file that holds the `unsafe` block. Both call sites that reach
      it — `edit` and `commit_tick` — are covered by the selection. Nothing in the module or in the
      selected tests touches Tokio or `midir`. The crate links ALSA through `midir` and a
      multi-threaded runtime, neither of which Miri can execute, but Miri interprets what runs, so a
      dependency no selected test calls never becomes a problem. The filter and this reasoning are
      recorded.
- [x] The job is manually triggered and is not a required check. It does not run on pull requests.
      `.github/workflows/miri.yml` declares `workflow_dispatch:` as its whole trigger, and no mise
      task calls `mise run miri`. Both facts are pinned by `scripts/check-tooling-contract.sh`.
- [x] The contract wording is checked against `verification-gaps/12`, which decided the contract
      stops *requiring* Miri while saying it is the tool this gate would prefer and should be run
      deliberately. This issue provides the deliberate path and must not turn it back into a
      requirement. `docs/tooling.md` states the deliberate, non-gating framing and cites the issue;
      the mise task and the workflow both carry it in comment. Proposed CLAUDE.md wording is in the
      handover rather than applied here.
- [x] `actionlint`, `zizmor`, and `bash scripts/check-tooling-contract.sh` are run and their results
      recorded, since this touches a workflow and a mise task. All three pass, as does
      `bash scripts/tests/check-tooling-contract.sh`. The new per-workflow rules are not exercised by
      that meta-suite, because its fixture copies only the workflows it names and `miri.yml` is not
      among them; adding a mutation case there was out of this change's file scope.

## Comments

This is the safety half of the effort, and it is small on purpose. The workspace holds exactly one
`unsafe` block — the in-place ASCII byte write in the Source model — and the clippy denials already
check that its invariant is written down. Miri is what checks whether the invariant holds.

The tests that reach it are the ones `02` also measures.

`nextest` is supported under Miri and gives each test its own interpreter context, which is what
makes leak detection at termination meaningful per test rather than per binary.
