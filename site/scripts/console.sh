#!/usr/bin/env bash
# Puts a web console built on its own into `site/public/play/`; the site never compiles it. npm runs
# this as `prebuild`, from `site/`, so `npm run build` there is the whole site build and is
# Cloudflare's build command. `CONSOLE_WEB` names a built console's directory, relative to `site/`,
# such as `../target/console-web` from `mise run console_web_build`, and replaces the console there.
# Without it, a console already in `public/play/` is kept, and otherwise the console published from
# `main` on the `console-web` release is downloaded; delete `public/play/` to fetch it again. The
# console is copied unchanged: its URLs are relative, so it serves from `/play/` under any
# `SITE_BASE`.
set -euo pipefail
if [[ -n "${CONSOLE_WEB:-}" ]]; then
  CONSOLE_WEB="$(cd "$CONSOLE_WEB" && pwd)"
fi
cd "$(dirname "$0")/.."
CONSOLE_URL="https://github.com/orcvs/orcvs/releases/download/console-web/console-web.tar.gz"

if [[ -n "${CONSOLE_WEB:-}" ]]; then
  rm -rf public/play
  mkdir -p public/play
  cp -R "$CONSOLE_WEB"/. public/play
elif [[ ! -f public/play/index.html ]]; then
  rm -rf public/play
  mkdir -p public/play
  if ! curl -fsSL "$CONSOLE_URL" | tar -xz -C public/play; then
    echo "Could not download the published console from $CONSOLE_URL." >&2
    exit 1
  fi
fi
if [[ ! -f public/play/index.html ]]; then
  echo "No web console in site/public/play/." >&2
  exit 1
fi
