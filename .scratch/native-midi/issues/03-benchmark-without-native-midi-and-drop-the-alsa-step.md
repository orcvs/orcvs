# 03 — Benchmark without native MIDI and drop the ALSA step

**What to build:** A benchmark run that stops building a system audio library it never calls. The Source benchmarks measure revision reads, Render Frames, and Language Map rebuilds; none of them touch MIDI, yet the benchmark jobs compile `midir` and link ALSA purely because they build the `orcvs` package. With `native-midi` off for the measured build, both benchmark CI jobs stop needing a native audio dependency installed at all.

Worth confirming rather than assuming: the benchmark command selects two packages at once, and how Cargo applies a default-features flag across a multi-package selection decides whether one command still does the job or the run has to be split.

**Blocked by:** 02 — Put midir behind a native-midi feature.

**Status:** ready-for-agent

- [ ] The standard local benchmark command builds `orcvs` with `native-midi` disabled, and neither `midir` nor a system audio library appears in the benchmark build.
- [ ] Every benchmark identifier and the shape of its measurement are unchanged, so the stored series stays continuous across the change.
- [ ] Both benchmark CI jobs stop installing native audio dependencies.
- [ ] The tooling contract asserts the featureless benchmark command and no longer requires the native dependency step it currently demands.
- [ ] `mise run bench` passes and emits every measurement in the CI-compatible output format.
- [ ] The scoped Rust gates pass.
