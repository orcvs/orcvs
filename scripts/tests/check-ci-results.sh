#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"

# Exercise all job outcomes, especially cancellation and unexpected skips.
for event in pull_request merge_group push workflow_dispatch unknown; do
  for linux in success failure cancelled skipped; do
    for wasm in success failure cancelled skipped; do
      for macos in success failure cancelled skipped; do
        expected=1
        if [[ "$linux" == success && "$wasm" == success ]]; then
          case "$event:$macos" in
            pull_request:skipped|merge_group:success|push:success|workflow_dispatch:success) expected=0 ;;
          esac
        fi
        actual=0
        CI_EVENT="$event" LINUX_RESULT="$linux" WASM_RESULT="$wasm" MACOS_RESULT="$macos" \
          bash "$repo_root/scripts/check-ci-results.sh" >/dev/null 2>&1 || actual=$?
        if [[ "$actual" != "$expected" ]]; then
          echo "Unexpected result for $event: Linux=$linux WASM=$wasm macOS=$macos" >&2
          exit 1
        fi
      done
    done
  done
done
echo 'CI result combinations passed'
