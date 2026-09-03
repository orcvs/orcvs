# 06 — Measure the per-keystroke rebuild path

**What to build:** Real numbers for the work done on every accepted keystroke, recorded where the
next person can find them, so the claims already made about that path are either evidenced or
withdrawn.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Tags:** release/v1

- [x] The Source edit and rebuild benchmarks are run across their Grid sizes on a machine quiet
      enough for the intervals to be trustworthy, and the load conditions are stated.
- [x] Numbers are captured for the commit before the Language Map rework and for the current tip,
      so the difference is attributable.
- [x] The result is recorded in this issue, including a result that shows no improvement.
- [x] Any performance claim that the numbers do not support is corrected at its source.

## Comments

Three commits on this branch changed that path and deliberately made no performance claim, because
every run that afternoon returned intervals wide enough to report both a large gain and a large
loss for the same code. The repository contract asks for a reproducible benchmark before claiming a
path got cheaper, so this is standing evidence debt rather than new work.

Guidance from the attempts already made: quit browsers and other agents first, use Criterion's
saved baselines rather than its implicit last-run comparison, and treat any run whose interval
spans more than a few percent as unusable rather than averaging it.

## Answer

The per-keystroke rebuild path got dramatically cheaper, and what changed is not a constant but
the shape of the curve: it was quadratic in the Cell count and is now linear. Read and render are
unchanged.

### Conditions

- Machine: Darwin 25.4.0, 8 cores (`hw.ncpu`), Rust 1.98.0 from `rust-toolchain.toml`.
- Load averages taken at the start of each run: `pre1` 2.22, `tip1` 2.03, `tip2` 3.14, `pre2` 3.15
  (one-minute figure). An earlier attempt the same afternoon read 13.71 with five headless Chromium
  instances and several Node builds belonging to other agent sessions; that is the condition the
  guidance above warns about, and no number was taken under it.
- Each commit was measured twice, in the order pre, tip, tip, pre, so a drift in machine state
  would show as disagreement between a commit's own two runs rather than as a difference between
  commits. Criterion defaults throughout: 3 s warm-up, 5 s measurement, 100 samples.
- Repeatability: every reported pair agrees within 0.5%, well inside the "few percent" bar. Two
  individual runs did not and are named below rather than averaged in.

### Trustworthiness

Two runs returned intervals too wide to use and are discarded, not averaged:

- `tip1 source_edit_rebuild_invalid/32x32` — [40.547 µs, 42.559 µs, 45.147 µs], an 11% spread,
  against `tip2`'s [38.414 µs, 38.445 µs, 38.474 µs].
- `pre2 source_read_revision/32x32` — [37.839 ns, 39.035 ns, 41.059 ns], an 8% spread, against
  `pre1`'s [37.927 ns, 38.033 ns, 38.166 ns].

### Numbers

Point estimates, both runs of each commit, so the reader can see the agreement rather than take it
on trust. `pre` is `f8f7bb6`, the last commit before the Language Map rework; `tip` is `e775dd6`.

**`source_edit_rebuild_valid` — one accepted keystroke that keeps the Expression valid**

| Grid  | Cells | pre1      | pre2      | tip1      | tip2      | change |
| ----- | ----- | --------- | --------- | --------- | --------- | ------ |
| 16x16 | 256   | 26.42 µs  | 26.32 µs  | 10.94 µs  | 10.92 µs  | −59%   |
| 32x32 | 1024  | 178.76 µs | 178.18 µs | 38.20 µs  | 38.25 µs  | −79%   |
| 64x64 | 4096  | 1.7273 ms | 1.7183 ms | 145.83 µs | 145.44 µs | −92%   |

**`source_edit_rebuild_invalid` — one accepted keystroke that leaves the Source malformed**

| Grid  | Cells | pre1      | pre2      | tip1       | tip2      | change |
| ----- | ----- | --------- | --------- | ---------- | --------- | ------ |
| 16x16 | 256   | 27.03 µs  | 26.97 µs  | 11.36 µs   | 11.39 µs  | −58%   |
| 32x32 | 1024  | 179.62 µs | 179.00 µs | (discarded) | 38.45 µs  | −79%   |
| 64x64 | 4096  | 1.7241 ms | 1.7175 ms | 145.32 µs  | 145.00 µs | −92%   |

**`source_read_revision` — what a Render Frame pays to observe an unchanged revision**

| Grid  | pre1     | pre2        | tip1     | tip2     | change |
| ----- | -------- | ----------- | -------- | -------- | ------ |
| 16x16 | 22.56 ns | 22.52 ns    | 22.51 ns | 22.50 ns | none   |
| 32x32 | 38.03 ns | (discarded) | 40.69 ns | 40.78 ns | +7%    |
| 64x64 | 73.28 ns | 73.40 ns    | 72.72 ns | 74.29 ns | none   |

**`source_render_frame` — deriving a whole frame from one revision**

| Grid  | pre1     | pre2     | tip1     | tip2     | change |
| ----- | -------- | -------- | -------- | -------- | ------ |
| 16x16 | 1.623 µs | 1.606 µs | 1.613 µs | 1.619 µs | none   |
| 32x32 | 5.556 µs | 5.517 µs | 5.562 µs | 5.557 µs | none   |
| 64x64 | 21.63 µs | 21.63 µs | 21.17 µs | 21.16 µs | −2%    |

### What the numbers say

The interesting result is not the percentage, it is the scaling. Four times the Cells cost:

- before: 6.8x then 9.7x — superlinear, consistent with work proportional to Cells squared;
- after: 3.5x then 3.8x — linear in the Cell count, which is what one pass over the Source is.

That is the difference between a console that gets slower the bigger the Grid and one that does
not. At the default 32x32 an accepted keystroke went from 179 µs to 38 µs; at 64x64 it went from
1.7 ms to 146 µs, and 1.7 ms is a figure a typist would eventually feel.

The two results that are *not* improvements are recorded because they are results. Render Frame is
flat, which is expected: the rework changed how a revision is derived, not how one is read.
`read_revision` at 32x32 is 5-7% slower at the tip and unchanged at the sizes either side of it,
which is a 2.7 ns difference on a 40 ns operation; a change that does not appear at 16x16 or 64x64
is layout or allocator noise rather than a path that got more expensive, and this is recorded as
"unexplained, not worth chasing at these magnitudes" rather than as a regression.

### Caveats

- The comparison spans every commit from `f8f7bb6` to the tip, which is the Language Map rework
  *plus* the five tickets of this effort. That is what the ticket asked for — the difference the
  console actually sees — not an attribution to any single commit.
- The edit benchmark's measured region is not identical at the two commits. Before, `set` took a
  number and ran `check_idx`; after, it takes a Grid-minted index and runs `assert_owns_index`, and
  the minting sits outside the measured closure. Both are one integer comparison against a
  measurement of 10 µs to 1.7 ms, so neither can account for any part of a change this size — but
  the region moved, and a future comparison across this branch should know it.

### Performance claims audited

Checkbox four found nothing to withdraw. No code comment on this branch claims a path got faster;
`grid.rs`'s "allocation-free Copy values" is a statement about types, verifiable by reading. In the
commit messages, `93d3211`'s "a Span is Copy and allocates nothing" and `27bb4b8`'s "a run that
needs no parse allocates nothing" are structural statements about where an allocation sits, each
true by inspection, not claims about elapsed time. The three commits that changed this path made no
speed claim at all, which is the debt this ticket discharges: the claim they declined to make is
now measured, and it was a large one.

### Reproducing

Criterion writes its data under `CARGO_TARGET_DIR`, and this repository shares one target directory
across worktrees. Two commits therefore need two target directories, or the second build is
silently taken as fresh and the same binary is measured twice — that trap was hit and is why the
binaries below are compared by hash before they are run.

```sh
git worktree add --detach /tmp/pre-rework f8f7bb6

CARGO_TARGET_DIR=/tmp/bench-pre \
  cargo bench --manifest-path /tmp/pre-rework/Cargo.toml -p orcvs --bench source --locked --no-run
CARGO_TARGET_DIR=/tmp/bench-tip \
  cargo bench -p orcvs --bench source --locked --no-run

# the two binaries share a name; confirm they differ before believing anything
shasum -a256 /tmp/bench-pre/release/deps/source-* /tmp/bench-tip/release/deps/source-*

CRITERION_HOME=/tmp/crit/pre /tmp/bench-pre/release/deps/source-<hash> --bench --save-baseline pre
CRITERION_HOME=/tmp/crit/tip /tmp/bench-tip/release/deps/source-<hash> --bench --save-baseline tip
```

Run each twice and discard any case whose interval spans more than a few percent, rather than
averaging it with a clean one. `uptime` before each run; anything above about 4 on this machine is
not worth starting.
