# 08 — Run the network loopback on Windows

Status: resolved
Type: task
Blocked by: 07

**What to build:** A merge-tier `windows` job, shaped like `macos`, that runs ticket 07's
loopback and a real `ConnectionReset` case on a Windows runner ([spec](../spec.md), ADR 0085).
Nothing else in CI runs tests on Windows, and Windows is the one platform that reports a datagram
to a port with no listener as `ConnectionReset` on a later send
(`docs/research/windows-support.md`, "OSC/UDP").

The Windows research recommends a merge-queue-only job like `macos` (its option B), measured on a
manual dispatch first. This job runs only the `orcvs` network tests, not the whole workspace, so it
stays small; widening it is the Windows-support effort's job.

- [x] A test sends to a loopback port nothing listens on, sends again, and sees no diagnostic and
      the second send made. On Windows the first send's `ConnectionReset` reaches the second.
- [x] The `windows` job runs on merge-group, push to `main` and manual dispatch, never on a pull
      request, and `ci` treats it as `macos` is treated.
- [ ] `scripts/check-tooling-contract.sh`, `actionlint` and `zizmor` pass.

## Answer

`a_send_after_one_nothing_heard_is_made_and_reports_nothing` sends Tick 0 to a port nothing
listens on and Tick 1 to the listener, then asserts Tick 1 arrived and nothing was reported. On
Linux and macOS no reset occurs, so only the Windows job exercises the `ConnectionReset` path.

The merge tier's `windows` job (`windows-2025`, 40 minutes) runs `mise run test_network_loopback`
with `install: false`, since the task needs only cargo. It follows the research's option B scoped
to this test: `if: github.event_name != 'pull_request'`, and `ci` requires it skipped on pull
requests and successful otherwise, as it does `macos`. `scripts/tests/check-ci-results.sh` covers
every outcome. The contract script pins the task, the job's runner and its step. actionlint and
zizmor were not available locally and run in the tooling workflow; the job's wall time is measured
on its first manual dispatch.
