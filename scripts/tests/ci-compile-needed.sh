#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"

check() {
  local event="$1" files="$2" expected="$3" actual
  actual="$(CI_EVENT="$event" CHANGED_FILES="$files" bash "$repo_root/scripts/ci-compile-needed.sh")"
  if [[ "$actual" != "compile=$expected" ]]; then
    echo "expected compile=$expected for $event with: ${files//$'\n'/, }; got $actual" >&2
    exit 1
  fi
}

md_only=$'docs/tooling.md\n.scratch/ci-tiers/issues/07-skip.md\nREADME.md'
check pull_request "$md_only" false
check pull_request $'docs/tooling.md\nmise.toml' true
check pull_request $'.scratch/theming/examples/my-dark.toml' true
check pull_request $'console/src/lib.rs' true
check pull_request $'docs/notes.markdown' true
check pull_request '' true
for event in merge_group push workflow_dispatch; do
  check "$event" "$md_only" true
done
echo 'compile decisions passed'
