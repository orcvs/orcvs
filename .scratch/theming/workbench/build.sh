#!/usr/bin/env bash
# Rebuilds theme-workbench.html's embedded data:
#   - the built-in Themes, read from console/src/theme.rs;
#   - the specimen, theme-sample.orcvs as the real parser classifies it mid-run;
#   - the console's font.
#
#   build.sh [repo-root]        # default: /Users/tobyhede/psrc/orcvs
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
repo="${1:-/Users/tobyhede/psrc/orcvs}"
dumper="$here/specimen-dump"

cp "$repo/rust-toolchain.toml" "$dumper/"
# rustup reads the toolchain file from the working directory, so build from the dumper.
(cd "$dumper" && cargo build --quiet)
"$dumper/target/debug/specimen-dump" "$here/theme-sample.orcvs" 72 32 >"$here/specimen.json"
# A typed Bang is consumed by the next Tick, so the running frame shows none outside an Output
# Portal. Take each Bang the stopped frame holds where the running frame's Cell is empty.
STOPPED=1 "$dumper/target/debug/specimen-dump" "$here/theme-sample.orcvs" 72 32 >"$here/specimen-stopped.json"
python3 - "$here/specimen.json" "$here/specimen-stopped.json" <<'PY'
import json, sys
running, stopped = (json.load(open(p)) for p in sys.argv[1:3])
for row, still in zip(running, stopped):
    for x, cell in enumerate(still):
        if cell[1] == "b" and not cell[2] and not row[x][0] and not row[x][2]:
            row[x] = cell
json.dump(running, open(sys.argv[1], "w"))
PY
node "$here/extract.mjs" "$repo"
