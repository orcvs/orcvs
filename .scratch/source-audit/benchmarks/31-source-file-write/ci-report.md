# Paired CI result — source-file writing

[Workflow run 36416640889](https://github.com/orcvs/orcvs/actions/runs/36416640889) succeeded on all three independent Linux jobs. Source baseline: `0d1245ce855df3093fbf762ff4aad1e6cab5bec4`.

The single-conversion candidate meets the [predeclared decision rule](ci-plan.md). It reduced populated-fixture time by 19.1–26.5% within each runner, with all twelve adjacent populated comparisons favoring it. No empty/sparse median regressed by more than 5%. Recommend restoring the single whole-buffer conversion in a focused implementation follow-up, with the existing source-file correctness, allocation, and public-API gates. This experiment does not change the production writer.

| Job | CPU | Empty ratio | Sparse ratio | Populated ratio | Populated improvement |
|---|---|---:|---:|---:|---:|
| 1 | AMD EPYC 9V45 | 1.0012 | 0.9781 | 0.7929 | 20.7% |
| 2 | AMD EPYC 9V74 | 1.0135 | 0.9705 | 0.7345 | 26.5% |
| 3 | AMD EPYC 9V45 | 0.9467 | 0.9988 | 0.8095 | 19.1% |

Ratios are candidate/current; lower is faster. Each number divides median Criterion run means inside one job. Different absolute timings between runners are not evidence for either implementation.

Adjacent-pair ratios, in run order:

| Job | Empty | Sparse | Populated |
|---|---|---|---|
| 1 | 0.9849, 0.9705, 1.0162, 1.0145 | 0.9849, 0.9764, 0.9834, 0.9552 | 0.7738, 0.7844, 0.7931, 0.8422 |
| 2 | 1.0142, 1.0156, 1.0123, 1.0122 | 0.9704, 0.9706, 0.9704, 0.9729 | 0.6971, 0.7329, 0.7363, 0.7596 |
| 3 | 0.9789, 0.9525, 0.9548, 0.9343 | 1.0083, 0.9771, 0.9623, 1.0580 | 0.7935, 0.8379, 0.8076, 0.7880 |

## Method

Fixtures: a fixed 256x256 Grid, empty; sparse with character 1 at column 2 of row 0; populated from the existing `whole_grid_source` fixture. Setup is outside the timed loop. Each job built both variants on the pinned baseline with identical manifest, lockfile, toolchain and harness, in separate target directories: A applies the harness portion of `benchmark.patch` (`orcvs/benches/source.rs`), B additionally applies its `file.rs` and `model.rs` portions, which restore #174's writer and a private `Source::text` accessor. Build with `cargo bench --package orcvs --bench source --locked --no-run`. After both builds exited, each executable was warmed up once, then run in A B B A B A A B order with `--bench --warm-up-time 1 --measurement-time 3 --sample-size 60 --nresamples 10000 --noplot --save-baseline LABEL`. Every invocation passed exact empty/sparse output and all-fixture round-trip assertions before timing.

The driver script, raw samples and per-run Criterion estimates were not kept in the repository; the tables above carry every ratio the decision rule reads.

## Local comparisons

Two earlier local runs on one Apple M2 used the same patch, fixtures, flags and order. Against baseline `8728a90f`, the populated fixture favored the current writer (3.709 µs versus 3.965 µs, candidate +6.9%). Rebuilt against `0d1245ce`, which changes only Cargo.lock, it favored the candidate (5.505 µs versus 3.951 µs, candidate −28.2%): the current writer moved while the candidate held steady. Empty/sparse differences were small and overlapping in both. The cause of the reversal is unexplained, which is why the comparison moved to paired CI.

## Limits

These synthetic fixtures, uncontrolled hosted CPU scheduling/frequency, two Linux CPU models and one pinned compiler do not establish application-level impact or a universal platform winner. The decision rule is a practical consistency threshold, not a significance test. One sparse adjacent pair reached 1.0580 in job 3, while that job's sparse median ratio was 0.9988; individual measurements remain noisy.
