#!/usr/bin/env bash
set -euo pipefail

case "${CI_EVENT:?}" in
  pull_request) expected_macos=skipped ;;
  merge_group|push|workflow_dispatch) expected_macos=success ;;
  *) echo "Unsupported CI event: $CI_EVENT" >&2; exit 1 ;;
esac

# `changes` decides whether the workflow compiles. Only a Markdown-only pull
# request skips it, and then the WASM job is skipped rather than run; Linux still
# runs the contract and the roadmap planner, so it must still succeed.
if [[ "${CHANGES_RESULT:?}" != success ]]; then
  echo "CI failed: the compile decision did not succeed (changes=$CHANGES_RESULT)" >&2
  exit 1
fi
case "${COMPILE:?}" in
  true) expected_wasm=success ;;
  false)
    if [[ "$CI_EVENT" != pull_request ]]; then
      echo "CI failed: only a pull request may skip the compile, not $CI_EVENT" >&2
      exit 1
    fi
    expected_wasm=skipped
    ;;
  *) echo "CI failed: unexpected compile decision: $COMPILE" >&2; exit 1 ;;
esac

if [[ "${LINUX_RESULT:?}" != success ||
      "${WASM_RESULT:?}" != "$expected_wasm" || "${MACOS_RESULT:?}" != "$expected_macos" ]]; then
  echo "CI failed: Linux=$LINUX_RESULT WASM=$WASM_RESULT (expected $expected_wasm) macOS=$MACOS_RESULT (expected $expected_macos)" >&2
  exit 1
fi
