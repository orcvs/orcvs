#!/usr/bin/env bash
set -euo pipefail

# Prints `compile=true` or `compile=false` for `$GITHUB_OUTPUT`.
#
# Only a pull request whose diff is Markdown alone skips the compile. No crate
# compiles or reads a `.md` file, so such a diff cannot change what cargo builds
# or tests; the tooling contract and the roadmap planner do read Markdown, and
# `check_docs` still runs them. Every other event, and any diff the script
# cannot read, compiles.
#
# `CHANGED_FILES` is the diff, one path per line.

if [[ "${CI_EVENT:?}" != pull_request || -z "${CHANGED_FILES:-}" ]]; then
  echo compile=true
  exit 0
fi

while IFS= read -r path; do
  [[ -z "$path" ]] && continue
  if [[ "$path" != *.md ]]; then
    echo compile=true
    exit 0
  fi
done <<< "$CHANGED_FILES"

echo compile=false
