# 30 — Let every pushed console run finish

**What to build:** Every pushed commit's Console web run finishes, as
[ADR 0101](../../../docs/adr/0101-every-site-build-takes-the-console-its-commit-builds.md) states
("No push cancels another's run"). Today the `build` job's concurrency group in
`.github/workflows/console-web.yml` is keyed on the archive with `cancel-in-progress: false`, and
GitHub keeps only the newest pending run in a group: a third push of the same key replaces the
second push's pending run. The archive is still published by the run that replaced it, so the
console is not lost, but the replaced run reports `cancelled`.

**Status:** needs-triage

- [ ] Decide whether a replaced pending run of the same archive breaks ADR 0101's rule. If it does,
      change the workflow so no pending run is replaced; if it does not, amend ADR 0101 to say so.
- [ ] `docs/tooling.md` no longer records this as a divergence.
