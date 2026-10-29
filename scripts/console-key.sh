#!/usr/bin/env bash
# Prints the key of the web console the checked-out commit builds: one hash over the git objects
# of the paths in `.github/workflows/console-web.yml`'s path filter, which are the paths the console
# is built from. Two commits share a key exactly when they build the same console. The workflow
# publishes each console under its key, and the site's production build waits for its own
# (ADR 0093).
set -euo pipefail
cd "$(dirname "$0")/.."
# Each entry is a file or a directory followed by `/**`. Anything else, or no entry at all, would
# key the console by the wrong paths, so it fails instead.
mapfile -t paths < <(awk '/^    paths:$/ { list = 1; next } !/^      - / { list = 0 } list' \
  .github/workflows/console-web.yml | sed -E "s/^      - '([^']+)'$/\1/; s|/[*][*]$||" | sort -u)
if (( ${#paths[@]} == 0 )) || printf '%s\n' "${paths[@]}" | grep -qE "[*?'\"]"; then
  echo "console-web.yml's path filter is not a list of quoted files and directories." >&2
  exit 1
fi
git ls-tree HEAD -- "${paths[@]}" | git hash-object --stdin
