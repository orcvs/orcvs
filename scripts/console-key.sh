#!/usr/bin/env bash
# Prints the key of the web console the checked-out commit builds: one hash over the git objects
# of the paths below, which are the paths the console is built from. Two commits share a key
# exactly when they build the same console. `.github/workflows/console-web.yml` publishes each
# console under its key, and every Cloudflare build of the site waits for its own (ADR 0093,
# ADR 0101). A path the console build reads that is missing here lets two different consoles share
# a key.
set -euo pipefail
cd "$(dirname "$0")/.."
paths=(
  console
  orcvs
  lang
  synth
  examples
  scripts/stage-synth.sh
  scripts/console-key.sh
  .cargo/config.toml
  Cargo.toml
  Cargo.lock
  rust-toolchain.toml
  mise.toml
  .github/workflows/console-web.yml
)
git ls-tree HEAD -- "${paths[@]}" | git hash-object --stdin
