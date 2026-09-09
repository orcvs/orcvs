#!/usr/bin/env bash
set -euo pipefail

case "${CI_EVENT:?}" in
  pull_request) expected_macos=skipped ;;
  merge_group|push|workflow_dispatch) expected_macos=success ;;
  *) echo "Unsupported CI event: $CI_EVENT" >&2; exit 1 ;;
esac

if [[ "${LINUX_RESULT:?}" != success || "${WASM_RESULT:?}" != success ||
      "${MACOS_RESULT:?}" != "$expected_macos" ]]; then
  echo "CI failed: Linux=$LINUX_RESULT WASM=$WASM_RESULT macOS=$MACOS_RESULT (expected $expected_macos)" >&2
  exit 1
fi
