# Issue audit — 2026-09-29

At `cad296df` the tracker held 425 issues: 60 open and 365 settled (resolved, wontfix, superseded
or obsoleted). The preceding audit commit closed 14 of the 60. This pass audited the **46 left
open**, plus independently rechecked all **14 closures**. **47 issues remain open** after reopening
console-testing/04. It is not a fresh verification of the 365 issues settled before the audit.

Code baseline: `cad296df6e0fbcb3a8b325dddc557bf52cda1404`, confirmed as `origin/main` through a
remote query during this audit. Starting branch HEAD: `c34bccb64f791b393ca9bb4a3a9d4c6dfd03a95e`.
That commit changes issue documents only. Existing untracked performance skill files and
skills-lock.json were left untouched.

## Findings and corrections

1. **Reopened console-testing/04.** A real App harness is implemented, but all-four-edge arrow
   clamping, ordinary Source Backspace and ignored Source-focused key/release acceptance remain
   untested through it. The earlier audit explicitly acknowledged gaps and still resolved it.
   Existing Space and Delete tests receive credit; no duplicate tests are requested.
2. **Corrected dependency blocker metadata.** dependency-duplicates/02 and /03 included winit
   versions in their Blocked by titles. The roadmap interpreted 0.30/0.31 as nonexistent local
   issues 30/31. Metadata now contains only actual issue references. The upstream work remains open.
3. **Corrected allocation-reduction/02's scope.** Numeric operand double-decoding is already absent;
   Function spelling detection still builds and discards an error. Render Frames read cached
   analysis, so the cost is not reparsing every frame.
4. **Corrected spatial-tick-planning/07's scheduler account.** Advancing contact dependencies to
   intrinsically active owners are omitted regardless of direction; the rule is not limited to
   backward mover edges. ADR wording and the specified collision tests still remain.
5. **Credited the existing Theme inheritance test in theming/10.** It overrides GridBackground and
   asserts inherited panel_background remains unchanged. Other integration proof remains open.
6. **Qualified midi-port-ownership/08's timing premise.** Replacing a connection performs synchronous
   safety-reset sends on the actor that executes Ticks. A bounded installation scenario must be
   stated; an unconditional deadline guarantee for arbitrarily slow sends needs a design decision.

No implementation was changed. Existing release gaps remain real: Web MIDI is a fallback,
whole-console keyboard zoom is disabled, an odd-offset blank can be skipped by Output Portal
highlight fitting, and the candidate/capture/hardware evidence bundle is not complete.
These remain owned by their existing tickets.

## Evidence by partition

- [Language and spatial behavior](language.md): 11 open issues and one closure.
- [Console and Theme](console.md): 14 open issues and five closures.
- [Playback, MIDI and Source writing](playback.md): seven open issues and two closures.
- [Tooling, dependencies and release](tooling-and-release.md): 14 open issues and six closures.

Three delegated agents read issue criteria, implementation and tests independently. The coordinator
reviewed their reports and directly rechecked the disputed App-input coverage, Theme override
assertion, parser/scheduler edits, highlight loop, synchronous MIDI reset and stop gate. The
coordinator separately checked the locked dependency graph, roadmap parser, actual CI workflows,
branch protection, CI job summaries and individual browser/benchmark logs. Existing issue text was
not accepted as implementation evidence. Test discovery, source inference and actual execution are
identified separately in the reports.

## Completion evidence

Changed: seven existing issue files and five audit reports; one issue reopened, two blocker lines
repaired, implementation claims and one acceptance checkbox corrected. No source, dependencies,
features, ADRs or repository settings changed.

Tests added or updated: none. **28 focused Rust tests passed** (lang allocation 5; playback/MIDI/
Source allocation 17; console/Theme 6). Exact commands and skipped-test counts are in the partition
reports. All three initial runs and escalated retries failed before compilation in sccache with
Operation not permitted; rerunning with `RUSTC_WRAPPER=` succeeded.

Commands run:

- `node --test scripts/tests/roadmap.test.ts` — passed, 10 tests.
- `node scripts/roadmap.ts > /dev/null` — passed.
- `git diff --check` — passed.
- Full changed issue diff and every new report reviewed; all initial open issue identifiers covered.
- Remote read-only commands and authoritative run links are recorded in tooling-and-release.md.

Not run: crate/workspace clippy, full nextest, doctests, inspection and dependency audit — no Rust,
manifest, feature or presentation changes. `mise run check`, `mise run check_merge`,
`mise run test_wasm`, `mise run bench` and 256-case proptest — deferred to CI. Fresh existing CI
results were read, not rerun. Miri and physical/visual release acceptance were not performed.

Risks: no public API / unsafe / dependency / feature / concurrency / performance behavior changed.
Static inspection and focused tests do not prove all behavior, all interleavings or all platform
combinations. A passing historical CI run is scoped to its SHA and workload, not release approval.
