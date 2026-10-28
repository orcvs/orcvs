#!/usr/bin/env bash
# Prints the key of the web console the checked-out commit builds: one hash over the git objects
# of the paths in `.github/workflows/console-web.yml`'s path filter, which are the paths the console
# is built from. Two commits share a key exactly when they build the same console. The workflow
# publishes each console under its key, and the site's production build waits for its own
# (ADR 0093).
set -euo pipefail
cd "$(dirname "$0")/.."
mapfile -t paths < <(grep -oE "^      - '[^']+'$" .github/workflows/console-web.yml \
  | sed -E "s/^      - '//; s/'$//; s|/[*][*]$||" | sort -u)
git ls-tree HEAD -- "${paths[@]}" | git hash-object --stdin
