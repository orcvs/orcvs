# 06 — Run Miri deliberately against the Source model

**What to build:** Miri can be run against the Source model tests, by a command that exists and a
job someone can trigger, without becoming a check every pull request pays for. Since `ca215dae` the
Source model holds no `unsafe` block of its own: a Cell write goes through the safe
`SourceBuffer::write` and the standard library's `unsafe` beneath it, which is what Miri now
interprets.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] A task runs Miri over the Source model tests, installing the nightly toolchain it needs rather
      than changing the pinned channel. `mise.toml` `[tasks.miri]` installs `nightly` with the `miri`
      component and runs `cargo +nightly miri nextest run`; `rust-toolchain.toml` is untouched.
      **Written, never executed** — see the Status above.
- [x] The run is scoped by test filter, not by crate. `-E 'test(/^source::model::test::/)'` selected
      68 tests when this was written, confirmed then by `cargo nextest list`, all in the `orcvs` lib
      unit-test binary and all in `orcvs/src/source/model.rs`. The module has grown since; the count
      is not re-derived here. Both call sites that reach
      it — `edit` and `commit_tick` — are covered by the selection. Nothing in the module or in the
      selected tests touches Tokio. The crate links a multi-threaded runtime, which Miri cannot
      execute (`midir` and ALSA have since moved to `console`), but Miri interprets what runs, so a
      dependency no selected test calls never becomes a problem. The filter and this reasoning are
      recorded.
- [x] The job is manually triggered and is not a required check. It does not run on pull requests.
      `.github/workflows/miri.yml` declares `workflow_dispatch:` as its whole trigger, and no mise
      task calls `mise run miri`. Both facts are pinned by `scripts/check-tooling-contract.sh`.
- [x] The contract wording is checked against `verification-gaps/12`, which decided the contract
      stops *requiring* Miri while saying it is the tool this gate would prefer and should be run
      deliberately. This issue provides the deliberate path and must not turn it back into a
      requirement. `docs/tooling.md` states the deliberate, non-gating framing and cites the issue;
      the mise task and the workflow both carry it in comment. The CLAUDE.md wording has since been
      applied (`AGENTS.md:88-96`, which `CLAUDE.md` links to), but it still tells a reader to reach
      for Miri "when the byte write in `Source::set_source` … changes" — a write that is no longer
      `unsafe`.
- [x] `actionlint`, `zizmor`, and `bash scripts/check-tooling-contract.sh` are run and their results
      recorded, since this touches a workflow and a mise task. All three pass, as does
      `bash scripts/tests/check-tooling-contract.sh`. The new per-workflow rules are not exercised by
      that meta-suite, because its fixture copies only the workflows it names and `miri.yml` is not
      among them; adding a mutation case there was out of this change's file scope.

## Comments

This is the safety half of the effort, and it is small on purpose. When it was filed the workspace
held exactly one `unsafe` block — the in-place ASCII byte write in the Source model — and the clippy
denials checked that its invariant was written down. Miri is what checks whether an invariant holds.

The tests that reach it are the ones `02` also measures.

`nextest` is supported under Miri and gives each test its own interpreter context, which is what
makes leak detection at termination meaningful per test rather than per binary.

### Audit at cad296df — 2026-09-29

- The "one `unsafe` block" premise is gone. `ca215dae` moved the Cells into a copy-on-write
  `SourceBuffer`; `Source::set_source` (`orcvs/src/source/model.rs:473-482`) now calls the safe
  `self.inner.write(..)`. The only `unsafe` left under `orcvs/src` is the test-only counting
  allocator (`orcvs/src/lib.rs:39`, `#[cfg(all(test, ..))]`). `mise.toml:158-161` and
  `.github/workflows/miri.yml:16-25` already say shipped code holds no `unsafe`; the body now says so
  too.
- `AGENTS.md:88-96` (read through the `CLAUDE.md` symlink) still names "the byte write in
  `Source::set_source`" as the trigger for running Miri. That wording is outside this issue's files
  and is not changed here.
- The "68 tests" figure was a `cargo nextest list` count at the time; the module has grown, and no
  new figure is asserted without re-running the list.
- `midir` no longer links into `orcvs` (`orcvs/Cargo.toml` has no `midir`), so the ALSA clause is
  corrected.
- `gh run list --workflow miri.yml` returns no runs, so the ready-for-human step — dispatch it once
  and record the result — is still owed.

### Issue audit against d3fd1b27 — 2026-10-01

No Miri run exists (`gh run list --workflow miri.yml` is empty). `docs/tooling.md`'s claim that
`orcvs` links `midir` is corrected; `midir` is console-only (`console/Cargo.toml:131`). `AGENTS.md:97`
still names the byte write in `Source::set_source`. The `miri.yml` comment says the counting
allocators are outside the filter, but `orcvs/src/lib.rs:38-87` installs a `#[global_allocator]`
under `cfg(all(test, not(target_arch = "wasm32")))` with no `not(miri)`, in the same lib unit-test
binary the `source::model::test::` filter selects, so Miri interprets it. Correct the comment or add
`not(miri)` when dispatching.

### First dispatches — 2026-10-13

- Run 1, on `main` at `602f10b6`
  ([38330757550](https://github.com/orcvs/orcvs/actions/runs/38330757550)), failed after 3m 15s.
  `test_an_edit_classifies_the_cells_it_affects` and
  `test_an_overwritten_root_loses_its_reserved_turn` aborted inside `trace()`: the subscriber's
  timestamp reads the wall clock, which Miri's isolation rejects. This was an unsupported
  operation, not undefined behaviour. nextest's fail-fast stopped the run at 38 of 80 tests, and
  the 36 that ran passed.
- The fix: both crates' test `trace()` helpers log `without_time()` on every target, so native runs
  use the subscriber Miri sees. `docs/tooling.md` states the constraint.
- Run 2, on the fix branch
  ([38333501077](https://github.com/orcvs/orcvs/actions/runs/38333501077)), passed: 80 of 80
  selected tests in 6m 32s for the job. Setup took about 40s, the nightly install, `cargo miri setup`
  and build took about 1m 45s, and the tests took 4m 05s at 6 to 11s each.
- The 2026-10-01 audit's allocator point holds: the counting allocator in `orcvs`'s lib unit-test
  binary has no `not(miri)` and runs under the interpreter, and run 2 passes with it there. The
  `miri.yml` comment now says so instead of placing every counting allocator outside the filter.
- Whether Miri should run on pull requests is `07`.
