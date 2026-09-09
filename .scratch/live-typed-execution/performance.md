# Restore indexed Tick lookups

The manual benchmark at `8c8ba2c` measured a severe Tick scaling regression.
Its measurement step succeeded; the comparison action failed because the
workflow-dispatch payload contained no commit information. Those measurements
were not published to the shared baseline.

## Diagnosis

The investigation compared the same Source fixtures on one machine:

| Commit | Tick 64×64 | Tick 128×128 | Edges 64×64 | Edges 128×128 |
| --- | ---: | ---: | ---: | ---: |
| `64291cc` | 93.5 µs | 418 µs | 152 µs | 672 µs |
| `593613c` | 122 µs | 500 µs | 205 µs | 1.01 ms |
| `ab63016` | 360 µs | 3.93 ms | 583 µs | 9.62 ms |

The intermediate edge sample was noisy (0.84–1.26 ms). The severe regression
starts in the live scheduler rewrite, not the parser partition change or later
release-safe replacement preflight. A minimized fixture containing only
independent Additions writing into empty rows also reproduced the bad scaling.

Stage timing attributed the largest costs to dependency discovery and Bang
activation. The scheduler scanned unrelated computations for every output;
execution repeated that scan, and descendant lookup scanned all nodes and
walked their ancestors. Removing the replacement preflight alone did not fix
the regression. The earlier positioned-dependencies prototype and preceding
Rust scheduler had used indexed overlap lookup. The integrated prototype used
broad scans to demonstrate bounded semantics, without a performance claim.

## Implementation

Build sorted indexes of disjoint physical Function spellings, literal operands,
and all operand claims once per Tick. Binary-search the first possible overlap,
then visit only intersecting claims. Empty Source-boundary operands are excluded;
enclosing Expression extents are never indexed. The all-operands index retains
nested Function spellings, which must still block Bang activation.

Parser preorder makes each computation's subtree contiguous. Compute its end
once, then use that range for replacement ordering, safety preflight and
suppression. Use indexed root lookup and a worklist for potential Bang activation;
each activated root's subtree is visited once. Actual activation still requires
a current successful Bang result.

These changes retain fixed destinations, pending decoding, typed nested results,
original-anchor replacement checks, two-Cell admission, and atomic rejection.
Real dependency edges and affected descendants still cost work proportional to
their count. This is not a claim of constant-time arbitrary replacement.

## Verification

The existing `source_execute_tick` and `source_execute_tick_edges` benchmark
series reproduce the performance defect through `Source::execute`. Their fixture
contents and series names remain unchanged, preserving CI history. Comments now
explain that independent computations can also expose all-pairs lookup costs.

A focused comparison can be reproduced in separate checkouts, each with its own
`target/`, using:

```sh
RUSTC_WRAPPER= cargo bench --package orcvs --bench source --locked -- \
  'source_execute_tick(_edges)?/(64x64|128x128)' \
  --warm-up-time 1 --measurement-time 3 --sample-size 30
```

Two repeated instrumented runs of the indexed implementation produced identical
operation counts on the existing fixtures:

| Whole-plan operation | 64×64 | 128×128 | Growth |
| --- | ---: | ---: | ---: |
| Ordinary lookup comparisons | 6,260 | 30,096 | 4.81× |
| Edge-fixture lookup comparisons | 17,724 | 80,076 | 4.52× |
| Edge-fixture returned candidates | 456 | 1,800 | 3.95× |
| Edge-fixture descendant visits | 324 | 1,272 | 3.93× |

This confirms removal of the unrelated all-pairs scan on these fixtures.
These historical operation counts came from temporary instrumentation, which is
not shipped in the repository. Use the permanent benchmark command above for
comparisons of the delivered implementation; the table records the diagnostic
attribution, not output produced by the normal benchmark suite.

Instrumented dependency discovery at 128×128 fell from approximately 2.50 ms to
34–37 µs for the ordinary fixture, and from 3.75 ms to 86–171 µs for the edge
fixture. Potential Bang activation fell from 2.33 ms to 31–90 µs. These individual
stage samples support attribution, not an end-to-end latency guarantee.

Concurrent host builds made uninstrumented wall-clock samples unstable. The
short growth-assertion loop correctly failed on the original implementation
(11.17× ordinary growth), but also produced a noisy failure after indexing.
The longer command above completed; its variable samples are not used to claim
a precise overall speedup. Stable CI measurement remains necessary.

The existing Source/Tick regression seam covers replacement, suppression, partial
writes, empty boundary operands, Bang activation, failed suppliers and cycle
atomicity. No timing assertion was added to the correctness suite. An independent
read-only audit mapped all 26 acceptance cases and issues 01–10 against ADR 0034,
the integrated prototype and the indexed prototype, and found no actionable
divergence in the restored index implementation.

Full platform, feature and browser matrices, full-count property tests, and the
authoritative benchmark comparison remain deferred to CI. Parser allocation
optimization and the manual comparison action's payload error are separate work.

## Completion evidence

Changed: indexed physical overlap queries, preorder subtree ranges, direct root
lookup and worklist activation; benchmark comments and this profiling record.

Tests added or updated: no correctness tests added; existing Source/Tick tests
and benchmark fixtures retained. Temporary diagnostic instrumentation is excluded
from the delivered implementation.

Commands run:

- `cargo fmt --all -- --check` — passed.
- `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --package orcvs --locked -E 'test(source::tick)' --status-level fail` — 43 passed.
- `RUSTC_WRAPPER= cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --workspace --locked --status-level fail` — 505 passed.
- `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --workspace --tests --features persistence --locked --status-level fail` — 509 passed.
- `RUSTC_WRAPPER= cargo test --workspace --doc --locked` — 9 passed.
- `node --test scripts/tests/roadmap.test.ts` — 10 passed.
- `node scripts/roadmap.ts > /dev/null` — passed.
- `git diff --check` — passed.
- Focused benchmark command above — completed; noisy latency samples limited to diagnostic use.
- Temporary instrumented benchmark, repeated during diagnosis — identical operation counts; both comparison-growth checks below 6× passed.

Not run: broad platform/feature/WASM/browser matrix, full 256-case proptest and
authoritative benchmark comparison — deferred to CI. No public API, unsafe,
dependency, feature or platform change introduces additional conditional gates.

Risks: the indexes rely on the Parser's physical ownership and preorder, verified
by code review and exercised by the Source tests. Real dense dependencies and
large replacement subtrees can still be expensive. Host contention prevents a
precise end-to-end latency claim from these local measurements.
