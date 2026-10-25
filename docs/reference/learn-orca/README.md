# Learn Orca reference copy

A copy of [Learn Orca](https://metasyn.srht.site/learn-orca/), an interactive tutorial for
[Orca](https://100r.co/site/orca.html) written by [metasyn](https://metasyn.pw). It is reference
material for Orca's behaviour and vocabulary. It does not describe Orcvs, and nothing in it decides
an Orcvs language question; those are recorded in `docs/adr/`.

## Contents

- [learn-orca.md](learn-orca.md): the tutorial text.
- [examples/](examples/): the example patches the site loads into Orca from its tutorial, basics,
  setups and misc menus.

## Source

The site publishes from [learn-orca on sourcehut](https://git.sr.ht/~metasyn/learn-orca). The
files here come from the project's former GitHub repository,
[metasyn/learn-orca](https://github.com/metasyn/learn-orca), at commit
`e1fb777c` (2022-07-26), the last commit with content before that repository was emptied in favour
of sourcehut. The section headings of the published page matched this copy on 2026-10-25.

| Here | Upstream file |
| --- | --- |
| `learn-orca.md` | `sections.md`, with the operator diagrams from `js/main.js` |
| `examples/misc/simpleBeat.orca` | the `simpleBeat` example in `js/main.js` |
| `examples/**` (all other files) | `orca-examples/` |

## Licences

- The tutorial text and `examples/misc/simpleBeat.orca` are released into the public domain under
  the Unlicense; see [LICENSE](LICENSE).
- The other example patches are hundred rabbits'
  [orca-examples](https://github.com/hundredrabbits/orca-examples), copyright Devine Lu Linvega,
  under the MIT License; see [examples/LICENSE](examples/LICENSE).

## Changes in conversion

The tutorial's prose, tables, code blocks and links are unchanged, including links that point at
the former GitHub repository. Three things differ because the page builds them with JavaScript:

- Each operator diagram, an empty `<div data-operator>` that `js/main.js` fills with a grid of
  cells and hover tooltips, is a fenced grid followed by a list of each cell's tooltip. Empty
  cells are `.`.
- The `<span>` tags that colour the words "inputs" and "outputs" are removed, leaving the words.
- `simpleBeat.orca` is the example's string without its leading indentation and blank lines.

The site leaves out the OSC, UDP and benchmark patches from `orca-examples/`, and so does this
copy.
