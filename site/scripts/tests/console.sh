#!/usr/bin/env bash
# Tests `site/scripts/console.sh` and `scripts/console-key.sh` as Cloudflare's build runs them
# (ADR 0093), without the network. Each case is a throwaway repository holding the two scripts and
# the real `console-web.yml`, whose path filter decides the key. `curl` and `sleep` are stubbed on
# `PATH`: `curl` serves files from a directory standing in for the `console-web` release, and
# `sleep` counts the waits and can publish a console partway through them.
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../../.." && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

mkdir -p "$work/bin"
cat > "$work/bin/curl" <<'STUB'
#!/usr/bin/env bash
out=""
prev=""
for arg in "$@"; do
  [[ "$prev" == -o ]] && out="$arg"
  prev="$arg"
done
[[ "$prev" == https://github.com/orcvs/orcvs/releases/download/console-web/* ]] || exit 2
src="$FAKE_RELEASE/${prev##*/}"
[[ -f "$src" ]] || exit 22
cp "$src" "$out"
STUB
cat > "$work/bin/sleep" <<'STUB'
#!/usr/bin/env bash
count=$(( $(cat "$FAKE_SLEEPS" 2>/dev/null || echo 0) + 1 ))
echo "$count" > "$FAKE_SLEEPS"
if [[ -n "${FAKE_PUBLISH:-}" && "$count" == "$FAKE_PUBLISH_AFTER" ]]; then
  cp "$FAKE_PUBLISH" "$FAKE_RELEASE/"
fi
STUB
chmod +x "$work/bin/curl" "$work/bin/sleep"

repo() {
  rm -rf "$work/repo" "$work/release" "$work/sleeps"
  mkdir -p "$work/release"
  git init -q "$work/repo"
  git -C "$work/repo" config user.email test@example.com
  git -C "$work/repo" config user.name test
  git -C "$work/repo" config maintenance.auto false
  git -C "$work/repo" config gc.auto 0
  mkdir -p "$work/repo/scripts" "$work/repo/site/scripts" "$work/repo/.github/workflows" "$work/repo/console"
  cp "$repo_root/scripts/console-key.sh" "$work/repo/scripts/"
  cp "$repo_root/site/scripts/console.sh" "$work/repo/site/scripts/"
  cp "$repo_root/.github/workflows/console-web.yml" "$work/repo/.github/workflows/"
  echo 'fn main() {}' > "$work/repo/console/main.rs"
  echo '# Home' > "$work/repo/site/index.md"
  commit
}

commit() {
  git -C "$work/repo" add -A
  git -C "$work/repo" commit -qm change
}

key() {
  bash "$work/repo/scripts/console-key.sh"
}

# A console archive whose page says which console it is.
console() {
  local name="$1" label="$2"
  rm -rf "$work/console"
  mkdir -p "$work/console"
  echo "$label" > "$work/console/index.html"
  tar -czf "$work/$name" -C "$work/console" .
}

build() {
  (cd "$work/repo/site" && PATH="$work/bin:$PATH" FAKE_RELEASE="$work/release" FAKE_SLEEPS="$work/sleeps" "$@" bash scripts/console.sh)
}

served() {
  cat "$work/repo/site/public/play/index.html"
}

fail() {
  echo "$1" >&2
  exit 1
}

# The key changes exactly when a path the console is built from changes.
repo
first="$(key)"
echo '# About' > "$work/repo/site/about.md"
commit
[[ "$(key)" == "$first" ]] || fail "a site-only commit changed the console key"
echo 'fn main() { loop {} }' > "$work/repo/console/main.rs"
commit
[[ "$(key)" != "$first" ]] || fail "a console commit kept the console key"

# A filter the key cannot read fails rather than keying the console by the wrong paths.
repo
sed -i "s|^      - 'Cargo.lock'$|      - '*.lock'|" "$work/repo/.github/workflows/console-web.yml"
commit
if key > /dev/null 2>&1; then fail "a glob in the path filter was keyed"; fi
repo
sed -i "s/^    paths:$/    branches-ignore: [gh-pages]/" "$work/repo/.github/workflows/console-web.yml"
commit
if key > /dev/null 2>&1; then fail "an empty path filter was keyed"; fi

# The production build waits for its own console, ignoring the rolling one already published.
repo
console console-web.tar.gz previous
cp "$work/console-web.tar.gz" "$work/release/"
console "console-web-$(key).tar.gz" this-commit
build env WORKERS_CI_BRANCH=main FAKE_PUBLISH="$work/console-web-$(key).tar.gz" FAKE_PUBLISH_AFTER=3 \
  || fail "the production build failed although its console was published while it waited"
[[ "$(served)" == this-commit ]] || fail "the production build served $(served), not its commit's console"

# A production build whose console never arrives fails, names the key, and stops waiting.
repo
console console-web.tar.gz previous
cp "$work/console-web.tar.gz" "$work/release/"
if output="$(build env WORKERS_CI_BRANCH=main 2>&1)"; then
  fail "the production build passed without its commit's console"
fi
grep -F "$(key)" <<< "$output" > /dev/null || fail "the failure does not name the key: $output"
# 45 tries with a wait between each.
[[ "$(cat "$work/sleeps")" == 44 ]] || fail "the build waited $(cat "$work/sleeps") times, not 44"

# Any other build takes the rolling console straight away.
repo
console console-web.tar.gz previous
cp "$work/console-web.tar.gz" "$work/release/"
build env WORKERS_CI_BRANCH=feature || fail "a preview build failed with a published console"
[[ "$(served)" == previous ]] || fail "a preview build served $(served), not the rolling console"
[[ ! -f "$work/sleeps" ]] || fail "a preview build waited"
