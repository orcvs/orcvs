# Epic — Implement the source audit

**Status:** resolved

**Reference:** Re-audited against `origin/main` at `67d28248069b1361de085b49e0c0008cc02d49fe` on 2026-09-25 by three delegated reviewers. Original source baseline: `199c3331`; implementation plan recorded on 2026-09-25.

## Outcome

Repair the confirmed correctness defects, make overload and concurrency guarantees explicit, reduce measured source and interpreter costs, and separate console responsibilities while preserving native and WASM behavior.

This epic organizes existing acceptance criteria into 24 implementation PR groups. PR numbers below are plan identifiers, not GitHub PR numbers. Child tickets remain authoritative for acceptance criteria and status: the epic being ready does not bypass a child's design decision, triage or blocker. The checkboxes track completed PR groups, not authorization to implement them all in one change.

## Scope

All 30 non-obsolete source-audit tickets belong to exactly one PR group. Source-audit/23 is obsolete and excluded. The related Playback fairness ticket joins PR 4; external prerequisites retain their existing owners. No render-cache implementation is added without current performance evidence.

## Playback correctness and resilience

| Done | PR | Scope | Issues | Sequencing and completion focus |
|---|---|---|---|---|
| [x] | 1 ([#144](https://github.com/orcvs/orcvs/pull/144)) | Fix stop admission | [#02](issues/02-close-the-tick-gate-race-between-answering-and-requesting-a-stop.md) | Delivered: count and gate in one atomic word, with the maintainer-approved two-outstanding-stops regression. |
| [x] | 2 ([#145](https://github.com/orcvs/orcvs/pull/145)) | Preserve BPM precision | [#01](issues/01-derive-the-tick-period-from-bpm-without-truncation.md) | Precise periods, rounding bounds and deadline tests. Independent of PR 1. |
| [x] | 3 ([#146](https://github.com/orcvs/orcvs/pull/146)) | Bound diagnostic retention | [#21](issues/21-bound-the-playback-diagnostics-queue.md) | Bound all diagnostic classes; record omissions and test mixed overload. |
| [x] | 4 ([#148](https://github.com/orcvs/orcvs/pull/148)) | Bound command admission and settle fairness | [#27](issues/27-bound-playback-command-admission.md), [playback-actor/11](../playback-actor/issues/11-record-what-keeps-the-biased-select-from-starving-the-clock.md) | Delivered: bounded mailbox and first-Tick backlog boundary. #27's nonblocking claim and playback-actor/11's backlog/slot terms are reconciled, as the audit follow-ups below record. |
| [x] | 5 ([#147](https://github.com/orcvs/orcvs/pull/147)) | Remove production test support | [#10](issues/10-move-orcvs-test-only-state-out-of-shipped-code.md) | Delivered: in-memory adapter gated, test-only execution recording removed, installation infallible, and no Playback test sleeps to manufacture elapsed time. The native `wait_until` harness poll is out of scope, as #10 records. |

## Source execution and performance

| Done | PR | Scope | Issues | Sequencing and completion focus |
|---|---|---|---|---|
| [x] | 6 ([#151](https://github.com/orcvs/orcvs/pull/151)) | Share unchanged Language Map rows | [#06](issues/06-reuse-unchanged-language-map-rows-across-revisions.md) | Remove deep clones, define revision behavior and update allocation evidence. |
| [x] | 7 ([#152](https://github.com/orcvs/orcvs/pull/152)) | Simplify Sequence-capability derivation | [#20](issues/20-derive-sequence-capability-in-one-place.md) | Linear in positioned entries, with explicit allocation and benchmark evidence; one scalar-width declaration; after [syntax-highlighting/11](../syntax-highlighting/issues/11-share-one-output-portal-derivation-with-the-scheduler.md). |
| [x] | 8 ([#153](https://github.com/orcvs/orcvs/pull/153)) | Cache dependency schedules | [#19](issues/19-cache-the-tick-schedule-per-language-map-revision.md) | Prefer after PR 6. Cover identical-write reuse and invalidation against fresh planning. #153 is merged (`f6599856`, which landed it with #151) and every criterion of #19 is met; the benchmark criterion was accepted on cross-runner CI evidence: Benchmark runs [36127565557](https://github.com/orcvs/orcvs/actions/runs/36127565557) (before) and [36134104362](https://github.com/orcvs/orcvs/actions/runs/36134104362) (after) show `source_execute_tick` 128x128 about 35%, 32% and 25% lower for the plain, `_edges` and `_portal_inputs` series. |
| [x] | 9 ([#154](https://github.com/orcvs/orcvs/pull/154)) | Plan outside Source locks | [#07](issues/07-plan-a-tick-outside-the-source-write-lock.md) | Prefer after PR 8. Snapshot planning with a consistently captured commit-validation identity, validated commit, stale-effect suppression and a bounded retry policy. |
| [x] | 21 ([#163](https://github.com/orcvs/orcvs/pull/163)) | Measure the Playback Tick path | [#28](issues/28-measure-the-playback-tick-path-in-ci.md) | After PR 9. A deterministic Tick series and per-Tick allocation record through `SourceCommander::execute`; records the baseline PR 10 is judged against. |
| [x] | 22 ([#162](https://github.com/orcvs/orcvs/pull/162)) | Read the Cells in place for Source File write and unsaved changes | [#29](issues/29-stop-copying-the-source-to-write-a-source-file-or-check-for-unsaved-changes.md) | Independent. One borrowed byte accessor on `Source`; `snapshot()` stays the owned form. |
| [x] | 10 ([#165](https://github.com/orcvs/orcvs/pull/165)) | Hold the Cells in a shared SourceBuffer | [#08](issues/08-remove-the-unsafe-byte-write-from-source.md) | After PR 21. Shared copy-on-write Cells remove the `unsafe` byte write and the whole-Cell copy per planning snapshot and per revision read; amend ADR 0057's cost paragraph. |
| [x] | 23 ([#166](https://github.com/orcvs/orcvs/pull/166)) | Pass the SourceBuffer through planning and the Language Map | [#30](issues/30-pass-the-source-buffer-through-planning-and-the-language-map.md) | After PR 10. Replace the runtime ASCII checks before parsing and operand reads with one checked view. |
| [x] | 24 ([#174](https://github.com/orcvs/orcvs/pull/174)) | Write a Source File through the SourceBuffer's checked text | [#31](issues/31-write-a-source-file-through-the-source-buffers-checked-text.md) | After PR 10; independent of PR 23. Remove `file::write`'s per-row ASCII check; judged on the `source_file` series. |

## Language implementation

| Done | PR | Scope | Issues | Sequencing and completion focus |
|---|---|---|---|---|
| [x] | 11 ([#156](https://github.com/orcvs/orcvs/pull/156)) | Narrow language APIs and diagnose invalid inputs | [#05](issues/05-diagnose-silent-drops-in-lang.md), [#09](issues/09-move-lang-test-only-api-behind-cfg-test.md) | Decide the fate of Interpreter::execute once; remove unused APIs and validate Jump width. |
| [x] | 12 ([#164](https://github.com/orcvs/orcvs/pull/164)) | Make operand declarations agree by construction | [#25](issues/25-make-an-operand-token-and-bind-agree-at-compile-time.md) | Design decision first; after PR 11. Compile-time token/bind agreement; preserve conversion input domains, idempotence, broadcasting and diagnostic precedence. |
| [x] | 13 ([#159](https://github.com/orcvs/orcvs/pull/159)) | Reduce interpreter work and allocations | [#24](issues/24-reduce-lang-per-tick-allocations.md) | After PR 11; preferably after PR 12. Measure the complete execute_function/binding path, including both Sequence clones; reconcile the criteria in [allocation-reduction/03](../allocation-reduction/issues/03-give-the-evaluation-stack-inline-storage.md). |

## Console behavior and structure

| Done | PR | Scope | Issues | Sequencing and completion focus |
|---|---|---|---|---|
| [x] | 14 ([#157](https://github.com/orcvs/orcvs/pull/157)) | Correct and test console integration behavior | [#03](issues/03-keep-the-panel-wake-up-across-a-function-reference-load.md), [#04](issues/04-make-the-browser-midi-output-agree-with-its-reporting-path.md), [#14](issues/14-drive-the-panel-layout-test-through-console-ui.md) | Open/repaint lifecycle, browser MIDI availability and real-console layout tests; establish protection before decomposition. |
| [x] | 15 ([#158](https://github.com/orcvs/orcvs/pull/158)) | Consolidate console test infrastructure | [#12](issues/12-move-console-inline-tests-into-sibling-modules.md), [#26](issues/26-share-console-test-setup-and-stop-polling-with-sleeps.md) | Move inline tests first, then share setup and replace native Playback polling. Preserve the test count at refactor start; 106 is only the audit baseline. |
| [x] | 16 ([#160](https://github.com/orcvs/orcvs/pull/160)) | Split console responsibilities | [#13](issues/13-split-console-ui-into-panel-modules.md) | After PRs 14–15. Separate file workflows, input routing, presentation and repaint ownership. Paint benchmark comparison recorded: Benchmark run [36222345284](https://github.com/orcvs/orcvs/actions/runs/36222345284) on #160 passed its compare-against-main step with no `paint_*` series alerting at the 150% threshold (its one alert, `source_read_revision/16x16`, stayed under the 300% fail threshold); the run failed only at its floor check, on `lang`'s `execute` (111 ns against the 100 ns floor then recorded), a floor #156 replaced with `execute_function`'s. The `main` push run [36222865145](https://github.com/orcvs/orcvs/actions/runs/36222865145) on `f34f1fb3` succeeded. |

## Themes and persistence

| Done | PR | Scope | Issues | Sequencing and completion focus |
|---|---|---|---|---|
| [x] | 17 ([#136](https://github.com/orcvs/orcvs/pull/136)) | Simplify Theme decoding | [#18](issues/18-make-toml-the-only-theme-representation.md) | Delivered by PR #136, which merged after the audit baseline. |
| [x] | 18 ([#169](https://github.com/orcvs/orcvs/pull/169)) | Settle colour-vision validation | [#11](issues/11-decide-the-fate-of-the-colour-blindness-simulation.md) | Decision first; preferably after PR 17. Integrate validation or remove the simulation from production. Decided: test-only check on the built-ins; removed from production. |
| [x] | 19 ([#170](https://github.com/orcvs/orcvs/pull/170)) | Discard undecodable stored Sources | [#22](issues/22-keep-an-earlier-refused-source-when-a-later-start-refuses.md) | Maintainer decision: the refused payload is discarded, not set aside. The start reports one error and opens the empty Grid, the next save overwrites the key, and the recovery key and notice are removed; #22 is wontfix. |

## Final cleanup

| Done | PR | Scope | Issues | Sequencing and completion focus |
|---|---|---|---|---|
| [x] | 20 | Reconcile comments with the finished implementation | [#15](issues/15-trim-lang-comments-to-their-durable-why.md), [#16](issues/16-trim-orcvs-comments-to-their-durable-why.md), [#17](issues/17-trim-console-comments-to-their-durable-why.md) | After each affected crate’s implementations and comment prerequisites. Permit separate crate PRs; include the delivered mailbox/fairness code and use caller needs, not function length, to size comments. The `lang` portion ([#15](issues/15-trim-lang-comments-to-their-durable-why.md)) is delivered by [#168](https://github.com/orcvs/orcvs/pull/168), the `orcvs` portion ([#16](issues/16-trim-orcvs-comments-to-their-durable-why.md)) by [#176](https://github.com/orcvs/orcvs/pull/176), and the `console` portion ([#17](issues/17-trim-console-comments-to-their-durable-why.md), with source-comments/04) by [#175](https://github.com/orcvs/orcvs/pull/175). |

## Sequencing

All 24 PR groups are complete. PR 23 merged as #166 (`84b76dd0`), PR 18 as #169, PR 19 was settled by discarding the refused payload (#170), and PR 24 is delivered by #174. PR 8 (#153) was accepted on cross-runner CI benchmark evidence, recorded in #19. PR 20's `lang` portion (source-comments/02 with #15) is delivered by #168, its `orcvs` portion (#16) by #176, and its `console` portion (source-comments/04 with #17) by #175.

Required ordering within this plan:

- PR 10 precedes PRs 23 and 24. PR 10 is delivered, and PR 23 followed it.
- Each crate’s portion of PR 20 follows that crate’s relevant implementation changes and external comment prerequisites; unrelated crates need not wait for each other.
- (satisfied) PR 1 before PR 4; PR 11 before PRs 12 and 13, with PR 12's design decision; PRs 14 and 15 before PR 16, tests moved before their setup was consolidated; PR 9 before PR 21, and PR 21 before PR 10; PR 18 after the colour-vision decision, recorded in #11.

Preferred ordering, not additional ticket blockers:

- (satisfied) PR 6 before PR 8: #151 and #153 landed together in `f6599856`, and #19 records 06 as an ordering preference rather than a blocker.
- (satisfied) PR 8 before PR 9 avoids changing schedule-cache ownership twice.
- (not followed, now moot) PR 12 before PR 13: #159 merged before #164; both are delivered.
- (satisfied) PR 17 before PR 18 avoids conflicting Theme-validation edits.

## External dependencies and existing work

- PR 7 followed [syntax-highlighting/11](../syntax-highlighting/issues/11-share-one-output-portal-derivation-with-the-scheduler.md), which was still open, so PR 7 delivered it first as a separate commit on the same branch. Both are resolved.
- PR 13 (#159) coordinated with [allocation-reduction/03](../allocation-reduction/issues/03-give-the-evaluation-stack-inline-storage.md), which owns operand-stack storage and remains ready-for-agent. Its criteria were first rewritten around `execute_function` on 2026-09-25 but still carried `operands.len() + 1`, a `MAX_OPERANDS + 1` bound and "if `execute` stays public" conditions after #156 and #159 landed; on 2026-09-27 they were aligned with the source (`Context::new(inputs, operands.len())`, a `MAX_OPERANDS` bound, no `execute` conditions) and with #24's allocation tests.
- PR 20 follows [source-comments/02](../source-comments/issues/02-apply-the-comment-rule-to-lang.md), [source-comments/03](../source-comments/issues/03-apply-the-comment-rule-to-orcvs.md), [source-comments/04](../source-comments/issues/04-apply-the-comment-rule-to-console.md). Manifest comment cleanup remains with [source-comments/05](../source-comments/issues/05-apply-the-comment-rule-to-scripts-and-configuration.md). 02 is resolved by #168, 03 by #171 and 04 by [#175](https://github.com/orcvs/orcvs/pull/175); 05 is ready-for-agent.
- PR 17 was delivered by PR #136, recorded in [#18](issues/18-make-toml-the-only-theme-representation.md).
- [#23](issues/23-refuse-invalid-grid-dimensions-through-the-error-path.md) remains obsoleted; do not reopen it as dimension-validation implementation work.

## Audit follow-ups before implementation or closure

These are epic-level reconciliation requirements from the `67d28248` audit. Child tickets remain authoritative; update their conflicting criteria explicitly before claiming completion. This epic update does not silently change their statuses or acceptance wording.

| Issue / group | Required reconciliation |
|---|---|
| #10 / PR 5 | Reconciled. #10's criterion covers sleeps that manufacture elapsed time for the Run Clock, which #147 removed. In `orcvs/src/playback.rs`, the native `wait_until` poll sleeps 1 ms against a wall-clock deadline for another thread's blocking adapter, and `idle` sleeps 1 ms of paused Tokio time so pending requests apply; neither moves the Run Clock, which reads `web_time::Instant`. Replacing the poll with a notification is separate work if wanted. |
| #27 and playback-actor/11 / PR 4 | #27 is reconciled: its design note now says no caller waits on the Playback task, a Tick or queue capacity, and acknowledges native mutex contention and that a caller replacing a pending connection drops it on its own thread. playback-actor/11 is reconciled: its body and criteria now state the budget as `BACKLOGS_BEFORE_A_DEADLINE = 64` backlogs of coalesced mailbox slots, and its 2026-09-27 comment reads the earlier triage bullets in those terms. The implementation is verified complete. |
| #07 / PR 9 | Reconciled: `PlanningSnapshot::capture` (`orcvs/src/source/planning.rs`) reads the `RevisionId` with the Grid, Cells and Language Map under one read guard (`Source::planning_snapshot`), and `commit` refuses a plan whose revision is no longer current; `a_snapshot_names_the_revision_its_contents_came_from` and `a_refused_plan_carrying_play_commands_delivers_none_of_them` pin it. |
| #20 / PR 7 | Reconciled: #20's criteria state cost linear in positioned entries and no allocation per Expression or entry (scratch at most once per call); `output_portal_reservations_allocate_the_same_blocks_for_every_shape` (`orcvs/src/source/language_map.rs`) pins the per-call block count and `source_render_frame_nested` (`orcvs/benches/source.rs`) measures growth with entry count. |
| #24 / PR 13 | Reconciled: `execute_function` moves operands onto the stack and the whole-value binds move the Sequence (`a_sequence_operand_is_consumed_without_copying_its_members`, `lang/tests/allocation.rs`); #24's allocation table separates removed copies from each answer's own allocation. The inline-attribute measurement covered the files #159 touched; the rest is split to lang-foundations/12. |
| #25 / PR 12 | Reconciled: tests in `lang/src/functions/numeric_conversion.rs` show both conversions accept Numbers and Notes, stay idempotent, broadcast elementwise (assembling nothing on failure) and keep diagnostic precedence; the `compile_fail,E0053` doctest in `lang/src/operand.rs` fails on a public Note token with a Number bind. |
| #12 and #26 / PR 15 | Reconciled: test counts are held to refactor start (`6ba42397`: 496 native console tests, 108 in the moved modules), unchanged after #158; `46d4d795` replaced #26's eight polling loops across six tests (PR 14 added one) with `engine_reaches`, and no console test sleeps. |
| #16 / PR 20 | Reconciled: its criterion and What-to-build prose size doc comments by what a caller needs, and its scope includes the Playback command mailbox and `BACKLOGS_BEFORE_A_DEADLINE` fairness code delivered in #148 as code to review, not prerequisites. |
| #08 and #09 / PRs 10–11 | #08 is reconciled: rescoped on 2026-09-26 to the shared SourceBuffer, with removal as the outcome and its cost measured against PR 21's baseline. #09 is reconciled: #156 met its criteria by removal and isolation — the unused APIs and `Interpreter::execute` were deleted, `lang`'s benchmark and allocation test moved to `execute_function`, and only `Stack::pop_value` became `cfg(test)`. Its resolved title is not realigned: its criteria carry the allowed outcome, and no integration test or benchmark depends on an API gated only by `cfg(test)`. |
| #22 / PR 19 | Superseded: a refused payload is discarded rather than set aside, so no loss sequence remains to pin. |

At the `67d28248` audit reference, #01, #02, #10, #18, #21 and #27 were resolved; playback-actor/11 was also resolved. #03 retained only its missing regression coverage. #23 remains obsolete. All other source-audit tickets were then outstanding, including explicit design decisions and measured evaluations. This paragraph records that audit; the tables above and each child ticket's Status line carry current status.

## 2026-09-28 closure reconciliation

A three-agent progress audit against `origin/main` `8728a90f` found two remaining closure mismatches. #16's follow-up corrects the Function-value test rationale and adds `relayed_bangs_reach_emission_refusals_at_the_right_and_top_edges`, proving that Jump relays reach both edges the old comment called impossible. [#177](https://github.com/orcvs/orcvs/pull/177) delivers this follow-up; its ticket records the 1180 passing tests and scoped checks.

## Audit evidence

- Three agents inspected every issue against `origin/main` `67d28248`; no implementation changes were made during the audit.
- At the `67d28248` audit, all 20 PR groups were checked: the 26 non-obsolete source-audit tickets are assigned exactly once and all local links resolve.
- `PROPTEST_CASES=32 cargo nextest run --package orcvs --locked -E 'test(playback::) | test(opts::)'` passed: 132 tests, 514 skipped.
- `node --test scripts/tests/roadmap.test.ts` passed all 10 tests; `node scripts/roadmap.ts > /dev/null` passed in the tickets worktree.
- Full workspace, WASM, dependency and performance checks were not rerun for the read-only audit. Source inspection and focused tests do not establish measured performance improvements.
- 2026-09-27 re-check at `2f955278` by six delegated reviewers: the 24 PR groups assign all 30 non-obsolete tickets exactly once; all 41 relative links resolve; the roadmap tests (10/10) and generation succeed; focused suites passed with `PROPTEST_CASES=32` — `orcvs` 678, `lang` 281 plus doctests, `console` 496. No implementation changes were made.

## Definition of done

- [x] Every PR group has a recorded disposition and links to its implementation PR or measured decision.
- [x] Every non-obsolete child ticket's acceptance criteria are satisfied and its status updated; the audit follow-ups above are reconciled explicitly, including PR 5’s recorded polling disposition. An explicitly permitted decision to retain an implementation is recorded with evidence.
- [x] Required external prerequisites are resolved or their scope is explicitly reconciled with the affected child tickets.
- [x] Correctness changes include meaningful regressions, including the single-word stop gate's two-outstanding-stops regression and stale-plan effect suppression.
- [x] Performance changes include reproducible benchmarks and the comparison evidence required by the child tickets; no unmeasured speedup is claimed.
- [x] Each implementation PR records the applicable crate, feature and platform verification. Native and WASM support are preserved.
- [x] Source-audit/23 remains excluded, all ticket references resolve, and the roadmap tests and generation succeed.

## Verification of epic and ticket edits

Run `node --test scripts/tests/roadmap.test.ts`, `node scripts/roadmap.ts > /dev/null`, and `git diff --check`. These validate the planning edits; they do not replace the implementation checks required by each child ticket.
