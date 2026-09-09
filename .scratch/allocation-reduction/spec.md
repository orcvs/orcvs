# Reduce the allocations the measured paths take

**Goal:** Remove the three heap allocations that `memory-verification` measured on paths that run on a clock, so the counts the new assertions bound fall rather than merely stop rising.

## Why

`memory-verification` built the counter and asked each path to allocate nothing. Three of them refused, and each refusal is recorded with its measurement in that effort's tickets. This effort is what those findings are for: measuring was the point of the last effort, and correcting is the point of this one.

None of the three is a defect in the sense of a wrong answer. Every one of them is work the path does not need to do, on a path that repeats:

- A Tick runs on a musical clock, once per Tick period, which `Bpm::delay_ms` puts under 20ms at the tempos a fast pattern reaches.
- A Render Frame re-reads the Source many times a second.
- A Language Map rebuild happens on every keystroke, and on every Tick that writes a Cell.

An allocation on any of those is paid at that rate. None of them is paid once.

## What is already true

The assertions that measured these are committed and passing, and every one of them is a zero, a ceiling, or a cross-multiplied ratio. Not one holds an absolute number. That is deliberate and it is what makes this effort cheap to verify: making any of these paths cheaper drives the numbers down and the existing tests still pass. A correction here needs no assertion edited.

The benchmarks that cover the same paths are also already in place — `execute`, `parse_source` and `parse_records` in `lang/benches/lang.rs`, and `source_edit_rebuild_valid`, `source_edit_rebuild_invalid` and `source_execute_tick` in `orcvs/benches/source.rs`. The contract asks for a benchmark behind a performance claim, and these are it. Running the comparison is the benchmark workflow's job, not a local one.

## Rules

**The measurement decides, not the reasoning.** Each ticket names the allocation count that must fall and the test that reports it. A change that reads better but does not move the count has not done the work.

**Do not weaken an assertion to land a change.** The assertions here are ceilings and ratios, so a correction passes them without edits. If one fails, the change made a path worse in some other dimension, and that is a finding.

**Correctness first.** Each of these paths has a large existing test suite behind it — `lang` and `orcvs` each hold hundreds of tests, and the property suites run over both. A correction that changes an answer is not a correction.

## Order

`01` first: it is the largest of the three by a wide margin, and it is the one whose fix is a design change rather than a local edit.

`02` and `03` are independent of `01` and of each other. `03` has a worked precedent in the repository — `05e4490` and `a215af7` did exactly this to two other structures — so it is the most mechanical of the three.

## Not in scope

Changing what any of these paths answers. Every ticket here is about what a path allocates while producing the answer it already produces.

The `Vec::with_capacity(previous.len())` growth transient recorded under `memory-verification/02` — a revision that adds one Language Unit to a row pays a doubling realloc on three vectors. It is a growth-policy effect rather than a shape, and `memory-verification/05` watches it in the published series. Reopen it only if the series shows it matters.

## Issues

- `issues/01-share-the-expressions-a-rebuild-carries-forward.md`
- `issues/02-test-a-function-spelling-without-building-an-error.md`
- `issues/03-give-the-evaluation-stack-inline-storage.md`
