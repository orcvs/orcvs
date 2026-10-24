#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"

# Exercise all job outcomes, especially cancellation and unexpected skips.
outcomes=(success failure cancelled skipped)
for event in pull_request merge_group push workflow_dispatch unknown; do
  for compile in true false ''; do
    for changes in "${outcomes[@]}"; do
      for linux in "${outcomes[@]}"; do
        for wasm in "${outcomes[@]}"; do
          for macos in "${outcomes[@]}"; do
            expected=1
            if [[ "$changes" == success && "$linux" == success ]]; then
              case "$event:$compile:$wasm:$macos" in
                pull_request:true:success:skipped | pull_request:false:skipped:skipped | \
                  merge_group:true:success:success | push:true:success:success | \
                  workflow_dispatch:true:success:success) expected=0 ;;
              esac
            fi
            actual=0
            CI_EVENT="$event" COMPILE="$compile" CHANGES_RESULT="$changes" LINUX_RESULT="$linux" \
              WASM_RESULT="$wasm" MACOS_RESULT="$macos" \
              bash "$repo_root/scripts/check-ci-results.sh" >/dev/null 2>&1 || actual=$?
            if [[ "$actual" != "$expected" ]]; then
              echo "Unexpected result for $event: compile=$compile changes=$changes Linux=$linux WASM=$wasm macOS=$macos" >&2
              exit 1
            fi
          done
        done
      done
    done
  done
done
echo 'CI result combinations passed'
