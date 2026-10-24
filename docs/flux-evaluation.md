# Grid refinement evaluation

Evaluated on 2026-10-13 at Orcvs base commit `455c2dbd` with exact mapping contracts.
Flux source: `b2f3e31e17727ffe08c2efee1e7aa0bc4f0a66f9`; its toolchain is
`nightly-2026-09-25`. Built with `RUSTC_WRAPPER= cargo xtask --rust-fixpoint install`;
the Rust fixpoint backend avoids installing the separate Haskell solver. Z3 is
available on this machine through Homebrew.

## Result and decision

Retain the narrow, opt-in experiment with exact mapping contracts. Do not expand
coverage or make it a CI gate on this evidence. Flux establishes the arithmetic
relationship for all inputs satisfying the contracts; the experiment found no
defect missed by the existing tests. It adds a complementary proof, not evidence
that the test suite is inadequate.

The original `index` contract specified only an in-range result. With that contract,
returning zero passed Flux. With the exact contract, it fails. The inverse contract
also specifies the exact quotient and remainder, rather than only valid coordinates.

| Implementation of `index` | Exact-contract Flux | Existing Grid tests |
| --- | --- | --- |
| `y * cols + x` | Passed | 38 passed |
| `x * cols + y` | Rejected: postcondition | Failed |
| `0` | Rejected: postcondition | Failed |
| `y * cols + x + 1` | Rejected: postcondition | Failed |

Each mutation was applied separately and restored. Nextest used its normal fail-fast
behaviour. The round-trip properties failed for all three mutations; no new Rust
tests or shipped test seams were necessary. Failure persistence was disabled to
avoid saving intentional counterexamples into the repository.

Incremental Flux mutation runs took 1.22–3.11 seconds; nextest commands took
22.14–23.08 seconds including recompilation. These are observations from one warm
checkout, not a performance benchmark or a general tool comparison.

Flux's correct-baseline summary was 56 functions checked, 673 trusted, one ignored,
and 17 constraints solved. The whole-crate totals must not be presented as proof
coverage: checking is scoped to Grid, and the surrounding crate is largely trusted.
Grid identity, ignored iteration, Source storage, and callers outside the checking
scope remain outside these arithmetic contracts. The trusted `NonZeroUsize` bridge
also remains an assumption. Normal builds retain all runtime guards.

## Reproduction

Install the pinned Flux source, with its declared nightly, rather than whatever
`main` points to:

```sh
git clone https://github.com/flux-rs/flux /tmp/flux-grid-evaluation
cd /tmp/flux-grid-evaluation
git checkout b2f3e31e17727ffe08c2efee1e7aa0bc4f0a66f9
RUSTC_WRAPPER= cargo xtask --rust-fixpoint install
```

From Orcvs, with Z3 available:

```sh
RUSTC_WRAPPER= RUSTFLAGS="-L $HOME/.flux" FLUXFLAGS="-Fstd-extern-specs" cargo flux -p orcvs
RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --package orcvs --locked -E 'test(grid::)'
```

For each mutation in the table, change only the `idx` expression in `Grid::index`,
run both commands, and restore the expression before the next mutation. Add
`PROPTEST_DISABLE_FAILURE_PERSISTENCE=1` to the nextest environment for intentional
failures. Check that Flux reports a refinement postcondition failure and nextest
reports assertion failures; a compilation, solver, or environment error does not
count as detecting the mutation. Restore the source and rerun both baselines.

To reproduce the bounds-only control, temporarily restore the original `index`
annotation from `455c2dbd` and change its expression to zero. Flux accepted this
control even with the stronger inverse contract present. Restore both afterwards.

## Maintenance boundary

Run Flux when the annotated Grid arithmetic or its contracts change. Keep it
optional while toolchain compatibility and unsupported Rust constructs require
accommodation. Revisit retention if it needs further changes outside Grid, loses
reproducibility, or stops checking the exact relationships. Expand it only for a
named invariant whose value justifies the new trusted assumptions and maintenance.

## Completion evidence

Changed: exact `index` and `position_at` refinement contracts, tooling instructions,
and this evaluation. Runtime implementations, features, and dependencies are unchanged.

Tests added or updated: none; existing Grid properties rejected every mutation.

Commands run on the restored implementation:

```text
cargo fmt --all -- --check — passed
git diff --check — passed
RUSTC_WRAPPER= RUSTFLAGS='-L /Users/tobyhede/.flux' FLUXFLAGS='-Fstd-extern-specs' cargo flux -p orcvs — passed
RUSTC_WRAPPER= CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo clippy --package orcvs --package console --all-targets --locked -- -D warnings — passed
RUSTC_WRAPPER= CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 PROPTEST_CASES=32 cargo nextest run --package orcvs --locked — passed: 945 tests, one skipped
RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --package orcvs --package console --locked — failed: disk full during compilation
RUSTC_WRAPPER= CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 PROPTEST_CASES=32 cargo nextest run --package orcvs --package console --locked — failed: disk full during compilation
```

The three intentional mutations each failed Flux with exit 101 and nextest with
exit 100. The bounds-only constant-zero control passed Flux. Initial environment
attempts also encountered an unusable sccache wrapper and sandbox network restrictions;
the completed checks above used a disabled wrapper and downloaded the missing crates.

Not run: console tests to completion — disk capacity prevented the test build even
with debug symbols disabled. `mise run check`, `mise run check_merge`,
`mise run test_wasm`, `mise run bench`, and proptest's 256-case default — Deferred to CI.
No pull request was opened, so the once-before-PR workspace and doctest gates were not run.

Risks: no runtime public-API, unsafe, concurrency, dependency, feature, or performance
change. Flux retains its separate nightly, solver assumptions, trusted boundaries,
and optional verification status; this is not a proof of the entire application.
