#!/usr/bin/env bash
# Puts a web console built on its own into `site/public/play/`; the site never compiles it. npm runs
# this as `prebuild`, from `site/`, so `npm run build` there is the whole site build and is
# Cloudflare's build command. `CONSOLE_WEB` names a built console's directory, relative to `site/`,
# such as `../target/console-web` from `mise run console_web_build`, and replaces the console there.
# Cloudflare's production build (`WORKERS_CI_BRANCH` is `main`) takes only the console its commit
# builds, published under that commit's console key, and waits up to 15 minutes for it before
# failing (ADR 0092). Any other build keeps a console already in `public/play/`, and otherwise
# downloads the console last published from `main`; delete `public/play/` to fetch it again. The
# console is copied unchanged: its URLs are relative, so it serves from `/play/` under any
# `SITE_BASE`.
set -euo pipefail
if [[ -n "${CONSOLE_WEB:-}" ]]; then
  CONSOLE_WEB="$(cd "$CONSOLE_WEB" && pwd)"
fi
cd "$(dirname "$0")/.."
RELEASE="https://github.com/orcvs/orcvs/releases/download/console-web"

# Replaces `public/play/` with the release's archive `$1`, or fails if it is not published.
fetch() {
  rm -rf public/play
  mkdir -p public/play
  curl -fsSL -o public/play/console.tar.gz "$RELEASE/$1" \
    && tar -xzf public/play/console.tar.gz -C public/play \
    && rm public/play/console.tar.gz
}

if [[ -n "${CONSOLE_WEB:-}" ]]; then
  rm -rf public/play
  mkdir -p public/play
  cp -R "$CONSOLE_WEB"/. public/play
elif [[ "${WORKERS_CI_BRANCH:-}" == main ]]; then
  key="$(bash ../scripts/console-key.sh)"
  # 45 tries 20 seconds apart: 15 minutes, inside Cloudflare's 20-minute build limit.
  for _ in $(seq 45); do
    fetch "console-web-$key.tar.gz" && break
    sleep 20
  done
  if [[ ! -f public/play/index.html ]]; then
    echo "The console this commit builds (key $key) was not published on the console-web release" \
      "within 15 minutes. Check the commit's Console web run, then retry this build." >&2
    exit 1
  fi
elif [[ ! -f public/play/index.html ]]; then
  if ! fetch console-web.tar.gz; then
    echo "Could not download the published console from $RELEASE/console-web.tar.gz." >&2
    exit 1
  fi
fi
if [[ ! -f public/play/index.html ]]; then
  echo "No web console in site/public/play/." >&2
  exit 1
fi
