# Paired CI decision rule for #31

Baseline: `0d1245ce855df3093fbf762ff4aad1e6cab5bec4` (origin/main when the experiment was prepared).

Three independent Linux jobs each build both variants, finish compilation, warm up both, then measure A B B A B A A B. A is the current writer; B is the single-conversion candidate from `benchmark.patch`. The harness checks exact empty/sparse output and round trips every fixture before timing. Each checkout owns its target directory. Outputs never enter the normal benchmark series.

Decide using ratios within each job, not absolute times between hosts. Before seeing CI results, define a meaningful difference as 5%. Favor the candidate only if all three jobs have a populated candidate/current median-run-mean ratio below 0.95, all four adjacent-pair ratios in every job are below 1.0, and neither empty nor sparse has a median ratio above 1.05 in any job. Otherwise classify performance as inconclusive and retain the current implementation pending stronger evidence. This is a practical consistency rule, not a statistical significance claim. Record individual pair ratios so drift and order effects remain visible.

Hosted-runner CPU affinity, frequency scaling, and other host activity remain uncontrolled. Linux results do not resolve the Apple Silicon build reversal or establish a cross-platform winner.
