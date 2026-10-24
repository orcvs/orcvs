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
# A pull request's checkout is the merge commit, so `HEAD^1` is the base branch
# and the diff is everything the pull request changes. `--no-renames` lists both
# sides of a rename: a Rust file renamed to `.md` removes a source file, and
# rename detection would report only the Markdown path it became.

if [[ "${CI_EVENT:?}" != pull_request ]] ||
   ! changed_files="$(git diff --name-only --no-renames HEAD^1 HEAD 2>/dev/null)" ||
   [[ -z "$changed_files" ]]; then
  echo compile=true
  exit 0
fi

while IFS= read -r path; do
  if [[ "$path" != *.md ]]; then
    echo compile=true
    exit 0
  fi
done <<< "$changed_files"

echo compile=false
