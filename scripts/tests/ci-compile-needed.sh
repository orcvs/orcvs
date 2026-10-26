#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
script="$repo_root/scripts/ci-compile-needed.sh"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

# Each case is a throwaway repository whose last commit is the change, so the
# script reads it through the same `git diff` it runs in CI.
repo() {
  rm -rf "$work/repo"
  git init -q "$work/repo"
  git -C "$work/repo" config user.email test@example.com
  git -C "$work/repo" config user.name test
  mkdir -p "$work/repo/src" "$work/repo/docs"
  echo 'fn main() {}' > "$work/repo/src/main.rs"
  echo '# Notes' > "$work/repo/docs/notes.md"
  git -C "$work/repo" add -A
  git -C "$work/repo" commit -qm base
}

commit() {
  git -C "$work/repo" add -A
  git -C "$work/repo" commit -qm change --allow-empty
}

check() {
  local event="$1" expected="$2" description="$3" actual
  actual="$(cd "$work/repo" && CI_EVENT="$event" bash "$script")"
  if [[ "$actual" != "compile=$expected" ]]; then
    echo "expected compile=$expected for $event, $description; got $actual" >&2
    exit 1
  fi
}

repo
echo more >> "$work/repo/docs/notes.md"
echo '# New' > "$work/repo/README.md"
commit
check pull_request false 'Markdown only'
for event in merge_group push workflow_dispatch; do
  check "$event" true "Markdown only"
done

repo
echo more >> "$work/repo/docs/notes.md"
echo 'x = 1' > "$work/repo/mise.toml"
commit
check pull_request true 'Markdown and a manifest'

repo
echo more >> "$work/repo/src/main.rs"
commit
check pull_request true 'a Rust source'

repo
mkdir -p "$work/repo/site/src"
echo 'export default {}' > "$work/repo/site/astro.config.mjs"
echo more >> "$work/repo/docs/notes.md"
commit
check pull_request false 'the site and Markdown'
check push true 'the site'

repo
mkdir -p "$work/repo/site"
echo '{}' > "$work/repo/site/package.json"
echo more >> "$work/repo/src/main.rs"
commit
check pull_request true 'the site and a Rust source'

repo
echo text > "$work/repo/docs/notes.markdown"
commit
check pull_request true 'a .markdown file'

repo
git -C "$work/repo" mv src/main.rs docs/main.md
commit
check pull_request true 'a Rust source renamed to Markdown'

repo
commit
check pull_request true 'an empty diff'

rm -rf "$work/repo"
git init -q "$work/repo"
check pull_request true 'no commit to diff against'

echo 'compile decisions passed'
