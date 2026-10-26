#!/usr/bin/env bash
# The build command Workers Builds runs, Cloudflare's GitHub integration, with `site` as its
# root directory: `bash cloudflare-build.sh`. Its build image has Node but no Rust or Trunk, so
# this installs the toolchain `rust-toolchain.toml` pins and the Trunk `mise.toml` pins, then
# runs `build.sh`, as the `site` workflow does. Cloudflare then deploys `dist/`.
set -euo pipefail
cd "$(dirname "$0")/.."

# The image picks Node from `site/.node-version`; it must be the version `mise.toml` pins.
node_pin=$(sed -n 's/^node = "\(.*\)"$/\1/p' mise.toml)
[[ "$(node --version)" == "v$node_pin" ]] || {
  echo "Node $(node --version) is not mise.toml's $node_pin; update site/.node-version" >&2
  exit 1
}

rustup_sha256=20a06e644b0d9bd2fbdbfd52d42540bdde820ea7df86e92e533c073da0cdd43c
rustup_init="$(mktemp -d)/rustup-init"
curl --proto '=https' --tlsv1.2 -fsSL -o "$rustup_init" \
  https://static.rust-lang.org/rustup/archive/1.28.2/x86_64-unknown-linux-gnu/rustup-init
echo "$rustup_sha256  $rustup_init" | sha256sum --check --quiet
chmod +x "$rustup_init"
"$rustup_init" -y --profile minimal --default-toolchain none
# shellcheck source=/dev/null
. "$HOME/.cargo/env"
rustup toolchain install

# The checksum is of mise.toml's Trunk; a Trunk bump fails here until it is updated.
trunk_version=$(sed -n 's|^"github:trunk-rs/trunk" = "\(.*\)"$|\1|p' mise.toml)
trunk_sha256=f2b4680cd239693a646a2795e4633c625328d7b2a044fbe749fa3a2fe9e7036b
trunk_archive="$(mktemp -d)/trunk.tar.gz"
curl --proto '=https' --tlsv1.2 -fsSL -o "$trunk_archive" \
  "https://github.com/trunk-rs/trunk/releases/download/v$trunk_version/trunk-x86_64-unknown-linux-gnu.tar.gz"
echo "$trunk_sha256  $trunk_archive" | sha256sum --check --quiet
tar -xzf "$trunk_archive" -C "$HOME/.cargo/bin" trunk

bash site/build.sh
