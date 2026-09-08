# Avoid allocating the common Parser pending stack

The [PR benchmark run at 7a5c7d2](https://github.com/orcvs/orcvs/actions/runs/34277731715)
failed on `parse_source`: 411 ns on the stored baseline versus 1,420 ns, or
3.45×. `parse`, `parse_invalid` and ordinary Tick execution also warned, but
none exceeded the 3× failure threshold. The indexed Tick repair restored its
scaling; this failure came from the Parser's per-parse work.

## Diagnosis and change

The live Parser used a freshly allocated `Vec` for pending operands, including
short and empty Source fragments. The first controlled experiment replaced only
that stack's storage, retaining reverse signature insertion and the same loop.
One million repetitions of the existing `parse_source` fixture consumed:

| Sample | Before, user CPU seconds | Inline pending stack, user CPU seconds |
| --- | ---: | ---: |
| 1 | 1.26 | 0.66 |
| 2 | 1.24 | 0.63 |
| 3 | 1.28 | 0.63 |

The paired fixed-work comparison isolates the change from wall-clock scheduling
delays. The original Criterion sample was badly affected by concurrent host work;
its reported percentage improvement is not used as evidence. A standard post-fix
Criterion run measured `parse_source` at 706–715 ns, `parse` at 179–183 ns and
`parse_invalid` at 61–68 ns. Local values do not substitute for CI comparison.

Production now holds the first 16 pending operands in an `ArrayVec` from the
existing dependency. A growable overflow `Vec` is used only after that fills.
Overflow is popped first; whenever it is occupied, the inline segment is full.
Pushes and pops therefore preserve the previous LIFO order through repeated
spills, drains and refills. Capacity 16 is an optimization, not a grammar limit.
No recursion or fixed maximum expression depth is introduced.

In this pending-stack repair, positioned record storage, parent identifiers, source
consumption, error recovery and decoding were unchanged. The experiment confirmed
that removing the pending stack allocation was sufficient for a large improvement; no additional record
layout or decoder changes were needed.

## Reproduction and coverage

The normal benchmark remains unchanged:

```sh
RUSTC_WRAPPER= cargo bench --package lang --bench lang --locked -- \
  parse --warm-up-time 1 --measurement-time 2 --sample-size 30
```

For fixed-work CPU comparison, apply the
[diagnostic benchmark patch](parser-cpu.profile.patch) in separate throwaway
checkouts before and after this repair. Each checkout must use its own `target/`.
Build with `RUSTC_WRAPPER= cargo bench --package lang --bench lang --locked --no-run`,
then use `/usr/bin/time -p` on the executable path Cargo reports. Alternate the
two executables three times. The patch changes only the benchmark entrypoint to
repeat the unchanged Source fixture one million times; it is not part of normal
CI and adds no production instrumentation.

`live_deep_sibling_computations_preserve_operand_order` checks two 33-leaf sibling
computations through Source parsing, scheduling and publication. The numerator
is 66, denominator 33, and the published answer must be 2. Reversing their order
would instead produce zero. This case spills, drains and refills the pending
stack. It passed before and after the optimization, locking down existing
semantics rather than inventing a correctness failure for a performance defect.
Existing long-expression round-trip properties and boundary/ownership tests also
passed after the change.

The independent read-only implementation/prototype auditor found no actionable
divergence. Its review specifically checked spill ordering, unbounded fallback,
preorder, parent claims, missing operands and error recovery.

Risks: a small bounded increase in native stack storage per parse; deeply nested
expressions still allocate overflow storage. No unsafe code, new dependencies,
features, public interface changes or semantic changes. Existing Tick timing
warnings remain separate from the parser benchmark's failing threshold.

## Local verification

- `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --package orcvs --locked -E 'test(live_deep_sibling)' --status-level fail` — 1 passed before the optimization.
- `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --package lang --package orcvs --locked -E 'test(long_expressions_roundtrip) or test(live_deep_sibling) or test(layout_preserves)' --status-level fail` — 3 passed after it.
- `cargo fmt --all -- --check` — passed.
- `RUSTC_WRAPPER= cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --workspace --locked --status-level fail` — 506 passed.
- `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --workspace --tests --features persistence --locked --status-level fail` — 510 passed.
- `RUSTC_WRAPPER= cargo test --workspace --doc --locked` — 9 passed.
- `node --test scripts/tests/roadmap.test.ts` — 10 passed.
- `node scripts/roadmap.ts > /dev/null` — passed.
- `git diff --check` — passed.
- `git apply --check .scratch/live-typed-execution/parser-cpu.profile.patch` — passed.
- Standard benchmark and paired CPU commands above — completed, measurements recorded above.

Not run locally: full feature/platform/WASM/browser matrices and 256-case
proptest — deferred to CI. The CI benchmark comparison is required to confirm
the repair against the stored baseline; local timings do not assert its verdict.

## Follow-up: eight inline Expression records

Expression now retains its first eight positioned records inline using the existing
`arrayvec` dependency, followed by an unbounded overflow `Vec`. Both borrowed and
consuming iteration preserve entry order; reverse token iteration crosses the
same boundary in reverse. Positions, parent identifiers and decoding are unchanged.
Eight is a provisional compromise to revisit when actual usage patterns emerge.
It is independent of the pending stack's capacity of sixteen.

Five rotated/reversed fixed-work CPU repetitions on the same host compared the
record capacities against commit `05e4490`. Median nanoseconds per parse were:

| Workload | Vec | Inline 4 | Inline 8 | Inline 16 |
| --- | ---: | ---: | ---: | ---: |
| Existing 16-row Source fixture | 894 | 633 | 648 | 790 |
| Balanced 3 records | 51 | 33 | 39 | 43 |
| Balanced 7 records | 129 | 81 | 69 | 75 |
| Balanced 15 records | 261 | 274 | 203 | 138 |
| Balanced 31 records | 441 | 464 | 463 | 399 |
| Balanced 63 records | 815 | 893 | 863 | 842 |

Eight improves the seven-record case while staying close to four on the existing
Source fixture. Sixteen helps fifteen-record Expressions but costs more on short
ones. Full Source edit/Tick timings were too variable to establish a winner;
separate-process setup subtraction even yielded invalid negative CPU estimates,
which were rejected. These measurements do not establish an application-wide
speedup. Larger inline objects may affect copying and cache behavior even when
available RAM is ample; this experiment did not isolate those effects.

The `parse_records/{3,7,15,31,63}` benchmarks retain the balanced workloads
for future comparisons, alongside the existing `parse_source` fixture. Run them
at each revision with the same toolchain and per-worktree build directory:

```sh
RUSTC_WRAPPER= cargo bench --package lang --bench lang --locked -- parse
```

Normal benchmark comparison remains CI's responsibility. The local diagnostic
scripts and raw samples are retained under `/tmp/orcvs-parser-capacities` and
`/tmp/orcvs-capacity-results` for this session, not as permanent repository assets.
Existing long-expression, positioned-boundary and deep-sibling tests cover the
unchanged behavior across overflow. No artificial failing correctness test was
introduced for this storage optimization.

### Follow-up completion evidence

Changed: Expression stores eight records inline with ordered, growable overflow.
The capacity choice and its measurement limitations are recorded above.

Tests added or updated: no correctness tests changed; existing boundary and deep
expression coverage passed before and after. Added balanced parser benchmarks
for 3, 7, 15, 31 and 63 records.

Commands run:

- `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --package lang --package orcvs --locked -E 'test(long_expressions_roundtrip) or test(live_deep_sibling) or test(layout_preserves)' --status-level fail` — passed, 3 tests before the change.
- `cargo fmt --all -- --check` — passed.
- `RUSTC_WRAPPER= cargo clippy --package lang --package orcvs --all-targets --locked -- -D warnings` — passed, including the new benchmark targets.
- `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --package lang --package orcvs --locked --status-level fail` — passed, 472 tests after the change.
- `node --test scripts/tests/roadmap.test.ts` — passed, 10 tests.
- `node scripts/roadmap.ts > /dev/null` — passed.
- `git diff --check` — passed; complete change diff reviewed.

Not run: CI benchmark comparison, full feature/platform/WASM/browser matrices and
256-case proptest — deferred to CI. Local capacity measurements above answer the
specific design question; no new full benchmark comparison was run as a gate.

Risks: performance remains workload dependent and each Expression has a larger
inline footprint. No public interface, unsafe, concurrency, dependency or feature
changes. Capacity can be revised without changing language semantics.
