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
