# Epic — Implement the source audit

**Status:** ready-for-agent

**Reference:** Re-audited against `origin/main` at `67d28248069b1361de085b49e0c0008cc02d49fe` on 2026-09-25 by three delegated reviewers. Original source baseline: `199c3331`; implementation plan recorded on 2026-09-25.

## Outcome

Repair the confirmed correctness defects, make overload and concurrency guarantees explicit, reduce measured source and interpreter costs, and separate console responsibilities while preserving native and WASM behavior.

This epic organizes existing acceptance criteria into 20 proposed implementation PRs. PR numbers below are plan identifiers, not GitHub PR numbers. Child tickets remain authoritative for acceptance criteria and status: the epic being ready does not bypass a child's design decision, triage or blocker. The checkboxes track completed PR groups, not authorization to implement them all in one change.

## Scope

All 26 non-obsolete source-audit tickets belong to exactly one PR group. Source-audit/23 is obsolete and excluded. The related Playback fairness ticket joins PR 4; external prerequisites retain their existing owners. No render-cache implementation is added without current performance evidence.

## Playback correctness and resilience

| Done | PR | Scope | Issues | Sequencing and completion focus |
|---|---|---|---|---|
| [x] | 1 ([#144](https://github.com/orcvs/orcvs/pull/144)) | Fix stop admission | [#02](issues/02-close-the-tick-gate-race-between-answering-and-requesting-a-stop.md) | Delivered: count and gate in one atomic word, with the maintainer-approved two-outstanding-stops regression. |
| [x] | 2 ([#145](https://github.com/orcvs/orcvs/pull/145)) | Preserve BPM precision | [#01](issues/01-derive-the-tick-period-from-bpm-without-truncation.md) | Precise periods, rounding bounds and deadline tests. Independent of PR 1. |
| [x] | 3 ([#146](https://github.com/orcvs/orcvs/pull/146)) | Bound diagnostic retention | [#21](issues/21-bound-the-playback-diagnostics-queue.md) | Bound all diagnostic classes; record omissions and test mixed overload. |
| [x] | 4 ([#148](https://github.com/orcvs/orcvs/pull/148)) | Bound command admission and settle fairness | [#27](issues/27-bound-playback-command-admission.md), [playback-actor/11](../playback-actor/issues/11-record-what-keeps-the-biased-select-from-starving-the-clock.md) | Delivered: bounded mailbox and first-Tick backlog boundary. Qualify the nonblocking claim as described in the audit follow-ups below. |
| [x] | 5 ([#147](https://github.com/orcvs/orcvs/pull/147)) | Remove production test support | [#10](issues/10-move-orcvs-test-only-state-out-of-shipped-code.md) | Delivered: in-memory adapter gated, test-only execution recording removed, installation infallible, and no Playback test sleeps to manufacture elapsed time. The native `wait_until` harness poll is out of scope, as #10 records. |

## Source execution and performance

| Done | PR | Scope | Issues | Sequencing and completion focus |
|---|---|---|---|---|
| [x] | 6 ([#151](https://github.com/orcvs/orcvs/pull/151)) | Share unchanged Language Map rows | [#06](issues/06-reuse-unchanged-language-map-rows-across-revisions.md) | Remove deep clones, define revision behavior and update allocation evidence. |
| [ ] | 7 | Simplify Sequence-capability derivation | [#20](issues/20-derive-sequence-capability-in-one-place.md) | Linear in positioned entries, with explicit allocation and benchmark evidence; one scalar-width declaration; after [syntax-highlighting/11](../syntax-highlighting/issues/11-share-one-output-portal-derivation-with-the-scheduler.md). |
| [ ] | 8 ([#153](https://github.com/orcvs/orcvs/pull/153)) | Cache dependency schedules | [#19](issues/19-cache-the-tick-schedule-per-language-map-revision.md) | Prefer after PR 6; reconcile #19’s hard blocker with the selected cache-key design. Cover identical-write reuse and invalidation against fresh planning. |
| [ ] | 9 | Plan outside Source locks | [#07](issues/07-plan-a-tick-outside-the-source-write-lock.md) | Prefer after PR 8. Snapshot planning with a consistently captured commit-validation identity, validated commit, stale-effect suppression and a bounded retry policy. |
| [ ] | 10 | Evaluate safe Source storage | [#08](issues/08-remove-the-unsafe-byte-write-from-source.md) | Prefer after PR 6. Measure a safe alternative and record removal or justified retention. |

## Language implementation

| Done | PR | Scope | Issues | Sequencing and completion focus |
|---|---|---|---|---|
| [ ] | 11 | Narrow language APIs and diagnose invalid inputs | [#05](issues/05-diagnose-silent-drops-in-lang.md), [#09](issues/09-move-lang-test-only-api-behind-cfg-test.md) | Decide the fate of Interpreter::execute once; remove unused APIs and validate Jump width. |
| [ ] | 12 | Make operand declarations agree by construction | [#25](issues/25-make-an-operand-token-and-bind-agree-at-compile-time.md) | Design decision first; after PR 11. Compile-time token/bind agreement; preserve conversion input domains, idempotence, broadcasting and diagnostic precedence. |
| [ ] | 13 | Reduce interpreter work and allocations | [#24](issues/24-reduce-lang-per-tick-allocations.md) | After PR 11; preferably after PR 12. Measure the complete execute_function/binding path, including both Sequence clones; reconcile the criteria in [allocation-reduction/03](../allocation-reduction/issues/03-give-the-evaluation-stack-inline-storage.md). |

## Console behavior and structure

| Done | PR | Scope | Issues | Sequencing and completion focus |
|---|---|---|---|---|
| [x] | 14 ([#157](https://github.com/orcvs/orcvs/pull/157)) | Correct and test console integration behavior | [#03](issues/03-keep-the-panel-wake-up-across-a-function-reference-load.md), [#04](issues/04-make-the-browser-midi-output-agree-with-its-reporting-path.md), [#14](issues/14-drive-the-panel-layout-test-through-console-ui.md) | Open/repaint lifecycle, browser MIDI availability and real-console layout tests; establish protection before decomposition. |
| [ ] | 15 | Consolidate console test infrastructure | [#12](issues/12-move-console-inline-tests-into-sibling-modules.md), [#26](issues/26-share-console-test-setup-and-stop-polling-with-sleeps.md) | Move inline tests first, then share setup and replace native Playback polling. Preserve the test count at refactor start; 106 is only the audit baseline. |
| [ ] | 16 | Split console responsibilities | [#13](issues/13-split-console-ui-into-panel-modules.md) | After PRs 14–15. Separate file workflows, input routing, presentation and repaint ownership. |

## Themes and persistence

| Done | PR | Scope | Issues | Sequencing and completion focus |
|---|---|---|---|---|
| [x] | 17 ([#136](https://github.com/orcvs/orcvs/pull/136)) | Simplify Theme decoding | [#18](issues/18-make-toml-the-only-theme-representation.md) | Delivered by PR #136, which merged after the audit baseline. |
| [ ] | 18 | Settle colour-vision validation | [#11](issues/11-decide-the-fate-of-the-colour-blindness-simulation.md) | Decision first; preferably after PR 17. Integrate validation or remove the simulation from production. |
| [ ] | 19 | Preserve refused stored Sources | [#22](issues/22-keep-an-earlier-refused-source-when-a-later-start-refuses.md) | Independent correctness fix: a later refusal must not replace the earlier recovery payload. |

## Final cleanup

| Done | PR | Scope | Issues | Sequencing and completion focus |
|---|---|---|---|---|
| [ ] | 20 | Reconcile comments with the finished implementation | [#15](issues/15-trim-lang-comments-to-their-durable-why.md), [#16](issues/16-trim-orcvs-comments-to-their-durable-why.md), [#17](issues/17-trim-console-comments-to-their-durable-why.md) | After each affected crate’s implementations and comment prerequisites. Permit separate crate PRs; include the delivered mailbox/fairness code and use caller needs, not function length, to size comments. |

## Sequencing

PRs 1–5 and 17 are verified complete. Start remaining correctness work with PR 19; PRs 11 and 14 can proceed independently. Before starting a PR, check child-ticket status and current source so already-landed work is not repeated.

Required ordering within this plan:

- PR 1’s stop-gate prerequisite for PR 4 is satisfied; both are delivered.
- PR 11 precedes PRs 12 and 13; PR 12 also needs its design decision settled.
- PRs 14 and 15 precede PR 16. Within PR 15, move tests before consolidating their setup.
- PR 18 needs the colour-vision decision settled.
- Each crate’s portion of PR 20 follows that crate’s relevant implementation changes and external comment prerequisites; unrelated crates need not wait for each other.

Preferred ordering, not additional ticket blockers:

- PR 6 before PR 8 coordinates map storage and cache identity. Row sharing is not intrinsically necessary for a correct cache key; reconcile #19’s existing hard blocker before choosing a different order.
- PR 8 before PR 9 avoids changing schedule-cache ownership twice.
- PR 6 before PR 10 makes the storage comparison easier to interpret.
- PR 12 before PR 13 avoids optimizing an operand-binding design about to change.
- PR 17 before PR 18 avoids conflicting Theme-validation edits.

## External dependencies and existing work

- PR 7 follows [syntax-highlighting/11](../syntax-highlighting/issues/11-share-one-output-portal-derivation-with-the-scheduler.md).
- PR 13 coordinates with [allocation-reduction/03](../allocation-reduction/issues/03-give-the-evaluation-stack-inline-storage.md), which owns operand-stack storage. Before implementation, rewrite its primary criteria around `execute_function` and stack allocations; its appended correction does not repair the old `execute` path and whole-Tick zero-allocation claims.
- PR 20 follows [source-comments/02](../source-comments/issues/02-apply-the-comment-rule-to-lang.md), [source-comments/03](../source-comments/issues/03-apply-the-comment-rule-to-orcvs.md), [source-comments/04](../source-comments/issues/04-apply-the-comment-rule-to-console.md). Manifest comment cleanup remains with [source-comments/05](../source-comments/issues/05-apply-the-comment-rule-to-scripts-and-configuration.md).
- PR 17 was delivered by PR #136, recorded in [#18](issues/18-make-toml-the-only-theme-representation.md).
- [#23](issues/23-refuse-invalid-grid-dimensions-through-the-error-path.md) remains obsoleted; do not reopen it as dimension-validation implementation work.

## Audit follow-ups before implementation or closure

These are epic-level reconciliation requirements from the `67d28248` audit. Child tickets remain authoritative; update their conflicting criteria explicitly before claiming completion. This epic update does not silently change their statuses or acceptance wording.

| Issue / group | Required reconciliation |
|---|---|
| #10 / PR 5 | Reconciled. #10's criterion covers sleeps that manufacture elapsed time for the Run Clock, which #147 removed. The native `wait_until` poll in `orcvs/src/playback.rs` waits against a wall-clock deadline for another thread's blocking adapter, and `idle` yields on the paused clock; neither manufactures elapsed time. Replacing the poll with a notification is separate work if wanted. |
| #27 and playback-actor/11 / PR 4 | Replace “Caller APIs never block” with no waiting on engine progress or queue capacity; acknowledge native mutex contention and synchronous destruction of superseded connections. Express fairness criteria in current backlog/slot terms. The implementation is verified complete. |
| #07 / PR 9 | Capture the validation identity with the snapshot contents. Current `SourceRevision` has no `RevisionId`; adding an internal planning snapshot is a valid solution. |
| #20 / PR 7 | Define complexity in positioned entries, distinguish no per-expression allocation from no allocation at all, and add benchmark/allocation acceptance evidence. |
| #24 / PR 13 | Cover cloning both when operands enter the stack and when whole-value binding extracts Sequences. Distinguish redundant copies from necessary result allocation. Scope inline-attribute measurement to touched/hot paths or split that review. |
| #25 / PR 12 | Preserve both numeric input types, conversion idempotence, broadcasting and diagnostic precedence. Compile-fail coverage must exercise the actual token/bind mismatch, rather than failing only because declarations are private. |
| #12 and #26 / PR 15 | Preserve tests relative to extraction/refactor start, allowing earlier PRs to add coverage. Correct #26’s count to six polling loops across five tests. |
| #16 / PR 20 | Remove the prose requirement that docs be shorter than functions; use what callers need. Include the delivered command mailbox and fairness changes in the final review without treating them as unresolved prerequisites. |
| #08 and #09 / PRs 10–11 | Align titles with allowed outcomes: measured safe-storage evaluation and removal or isolation of unused language APIs. Integration tests and benchmarks cannot use library APIs gated only by `cfg(test)`. |
| #22 / PR 19 | Pin the actual loss sequence: first refusal/save, another invalid primary payload, then second refusal/save. An ordinary successful save does not create the second refusal. |

At this reference, #01, #02, #10, #18, #21 and #27 are resolved; playback-actor/11 is also resolved. #03 retains only its missing regression coverage. #23 remains obsolete. All other source-audit tickets remain outstanding, including explicit design decisions and measured evaluations.

## Audit evidence

- Three agents inspected every issue against `origin/main` `67d28248`; no implementation changes were made during the audit.
- All 20 PR groups were checked: the 26 non-obsolete source-audit tickets are assigned exactly once and all local links resolve.
- `PROPTEST_CASES=32 cargo nextest run --package orcvs --locked -E 'test(playback::) | test(opts::)'` passed: 132 tests, 514 skipped.
- `node --test scripts/tests/roadmap.test.ts` passed all 10 tests; `node scripts/roadmap.ts > /dev/null` passed in the tickets worktree.
- Full workspace, WASM, dependency and performance checks were not rerun for the read-only audit. Source inspection and focused tests do not establish measured performance improvements.

## Definition of done

- [ ] Every PR group has a recorded disposition and links to its implementation PR or measured decision.
- [ ] Every non-obsolete child ticket's acceptance criteria are satisfied and its status updated; the audit follow-ups above are reconciled explicitly, including PR 5’s recorded polling disposition. An explicitly permitted decision to retain an implementation, such as PR 10's measured unsafe retention, is recorded with evidence.
- [ ] Required external prerequisites are resolved or their scope is explicitly reconciled with the affected child tickets.
- [ ] Correctness changes include meaningful regressions, including the single-word stop gate's two-outstanding-stops regression and stale-plan effect suppression.
- [ ] Performance changes include reproducible benchmarks and the comparison evidence required by the child tickets; no unmeasured speedup is claimed.
- [ ] Each implementation PR records the applicable crate, feature and platform verification. Native and WASM support are preserved.
- [ ] Source-audit/23 remains excluded, all ticket references resolve, and the roadmap tests and generation succeed.

## Verification of epic and ticket edits

Run `node --test scripts/tests/roadmap.test.ts`, `node scripts/roadmap.ts > /dev/null`, and `git diff --check`. These validate the planning edits; they do not replace the implementation checks required by each child ticket.
