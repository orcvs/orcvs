#!/usr/bin/env bash
# Prints the assets on the `console-web` release, one `<name> <created_at>` line each, oldest page
# first, reading every page so no keyed archive is missed. Prints nothing when the release does not
# exist yet, and fails on any other API error rather than reporting a console as unpublished
# (ADR 0101). Needs `gh` with `GH_REPO` set.
set -euo pipefail
err="$(mktemp)"
trap 'rm -f "$err"' EXIT
if ! id="$(gh api "repos/$GH_REPO/releases/tags/console-web" --jq .id 2> "$err")"; then
  if grep -q 'HTTP 404' "$err"; then
    exit 0
  fi
  cat "$err" >&2
  exit 1
fi
gh api --paginate "repos/$GH_REPO/releases/$id/assets?per_page=100" --jq '.[] | "\(.name) \(.created_at)"'
