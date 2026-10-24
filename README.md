# Orcvs

Orcvs is a grid-based environment for composing and performing compact musical expressions. You
write a program as two-character Functions in a 256 by 256 Grid of Cells, and on every Tick
Playback runs that program, writes its results back into the Grid, and sends MIDI.

Orcvs is an internal, pre-release application. The language is still being designed, so spellings
and behaviour change without a compatibility promise.

## Run it

You need Rust installed through `rustup`, which reads `rust-toolchain.toml`. On Linux the MIDI
backend also needs the ALSA development package, for example `libasound2-dev` on Debian or Ubuntu.
From the repository root:

```sh
cargo run
```

The first build takes several minutes, then the console opens as a desktop window. It saves the
running Source while it runs and when it closes, and opens on it next time, or on an empty Source
the first time.

**Help > Function Reference** replaces the running Source with one worked example of every
Function.

To see a result of your own, follow [Your first result](docs/first-result.md). It takes a few
minutes and needs no MIDI device.

## Learn the language

- [`CONTEXT.md`](CONTEXT.md) is the glossary: Source, Grid, Cell, Function, Tick, Portal and the
  rest.
- [`docs/adr/`](docs/adr/) records each language and architecture decision.
- [`console/assets/function_reference.orcvs`](console/assets/function_reference.orcvs) is the Source
  the Function Reference opens.

## Contribute

- [`AGENTS.md`](AGENTS.md) is the repository contract: sources of truth, Rust policy, and the
  checks each change runs.
- [`docs/tooling.md`](docs/tooling.md) describes the `mise` tasks and the CI tiers, including
  `mise run bench`.
- Work is tracked as Markdown under `.scratch/`; see
  [`docs/agents/issue-tracker.md`](docs/agents/issue-tracker.md).
