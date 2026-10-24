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
Function, asking first if the running Source has unsaved changes.

To see a result of your own, follow [Your first result](docs/first-result.md). It takes a few
minutes and needs no MIDI device.

## Changing the Bang colours

The console ships `grayscale-dark` and `grayscale-light` as its default Themes and switches between
them with the light/dark mode. To change one colour, add a custom Theme that inherits a default and
lists only what it changes. `source.bang` is the Bang glyph and `source.bang.border` its Cell border.

`~/.orcvs/themes/my-dark.toml`:

```toml
format = "orcvs-theme"
version = 1
name = "My Dark"
inherits = "grayscale-dark"

[style]
"source.bang" = "#FFAA00"
"source.bang.border" = "#FFAA00"
```

`~/.orcvs/themes/my-light.toml`:

```toml
format = "orcvs-theme"
version = 1
name = "My Light"
inherits = "grayscale-light"

[style]
"source.bang" = "#964B00"
"source.bang.border" = "#964B00"
```

Then select them by file name in `~/.orcvs/config.toml`, and restart the console:

```toml
[theme]
dark = "my-dark"
light = "my-light"
```

Every other property keeps the default's value. `console/src/theme.md` lists every property.

## Learn the language

- [`CONTEXT.md`](CONTEXT.md) is the glossary: Source, Grid, Cell, Function, Tick, Portal and the
  rest.
- [Function reference](docs/functions.md) lists every Function with its operands, what it does,
  and an example checked against the language.
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
