#!/usr/bin/env bash
# Builds the Built-in Synth's WebAssembly module and stages it, with its
# AudioWorklet and page scripts from `synth/web/`, beside the web console
# Trunk is building. Trunk runs it as a post-build hook
# (`console/Trunk.toml`) and names the staging directory.
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"

# Optimised whatever Trunk's profile: the module renders on the browser's
# audio thread, where a slow block is an audible gap.
cargo build --manifest-path "$root_dir/Cargo.toml" --package synth --lib \
  --target wasm32-unknown-unknown --release --locked
target_dir="$(cargo metadata --manifest-path "$root_dir/Cargo.toml" --format-version 1 --no-deps \
  | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')"
cp "$target_dir/wasm32-unknown-unknown/release/synth.wasm" \
  "$root_dir/synth/web/synth-worklet.js" \
  "$root_dir/synth/web/synth-host.js" \
  "${TRUNK_STAGING_DIR:?Trunk names the staging directory}/"
