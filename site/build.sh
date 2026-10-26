#!/usr/bin/env bash
# Builds the site around a web console built on its own; it never compiles the console. This is
# Cloudflare's build command (root directory `site/`), and `mise run site_build` and the `site`
# workflow run it too. `CONSOLE_WEB` names a built console's directory, such as the one `mise run
# console_web_build` writes to `target/console-web`; without it, the script downloads the console
# published from `main` on the `console-web` release. The console is copied into `public/play/`
# unchanged: its URLs are relative, so it serves from `/play/` under any `SITE_BASE`. Astro then
# copies `public/` into `dist/`.
set -euo pipefail
cd "$(dirname "$0")/.."
CONSOLE_URL="https://github.com/orcvs/orcvs/releases/download/console-web/console-web.tar.gz"

rm -rf site/public/play
mkdir -p site/public/play
if [[ -n "${CONSOLE_WEB:-}" ]]; then
  cp -R "$CONSOLE_WEB"/. site/public/play
elif ! curl -fsSL "$CONSOLE_URL" | tar -xz -C site/public/play; then
  echo "Could not download the published console from $CONSOLE_URL." >&2
  exit 1
fi
if [[ ! -f site/public/play/index.html ]]; then
  echo "No web console in site/public/play/." >&2
  exit 1
fi
cd site
npm ci
npm run build
