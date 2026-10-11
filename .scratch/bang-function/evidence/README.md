# Orca comparison probes

Run against the three upstream core files from a checkout of `hundredrabbits/Orca`:

```sh
node .scratch/bang-function/evidence/orca-probes.cjs /path/to/Orca/desktop/sources/scripts/core
```

The script executes the core in Node’s VM and replaces only `z` with a portless activation recorder. No scenario relies on native Z behavior; H, D, E, W, N, S and X retain their implementations. Output records initial rows, three committed frames, and `[frame, x, y]` probe activations. Compare with `observed.jsonl`; these are observations, not Orcvs expected results. The probe is a diagnostic runner, not an assertion-based test suite.

The recovered files used for `observed.jsonl` were attributed by the prior session to `223e45c8`. Their upstream revision has not been independently authenticated. SHA-256 fingerprints permit exact reproduction and make that limitation explicit:

- `operator.js`: `f9ce46da399453cd3b27d3261d3a9d6df4ba35e729b0190f6baf3401e27bb42f`
- `library.js`: `a0a20db009fcb090bfee81400cc791141ca2bccaaa62a2d06ac64140cabe8304`
- `orca.js`: `e1891f257680c3056bec3053fb50d8e26d5f1d5c7a3b1d09379314c1e8ef737f`

Before claiming revision-verified parity, retrieve the claimed commit and compare these hashes, then record the verified full commit. Differences in another revision require rerunning and reviewing the observations. Upstream source is not vendored here.
