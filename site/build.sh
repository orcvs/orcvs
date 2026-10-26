#!/usr/bin/env bash
# Builds the site, from the repository root, around a web console built on its own: `CONSOLE_WEB`
# names the console's directory, `target/console-web` by default, which `mise run
# console_web_build` writes and the `site` workflow fills from the `console-web` artifact. The
# console is copied into `site/public/play/` unchanged; its URLs are relative, so it serves from
# `/play/` under any `SITE_BASE`. Astro then copies `public/` into `site/dist/`. `mise run
# site_build` runs this, locally and in the `site` workflow's build job.
set -euo pipefail
CONSOLE_WEB="${CONSOLE_WEB:-target/console-web}"

if [[ ! -f "$CONSOLE_WEB/index.html" ]]; then
  echo "No web console in $CONSOLE_WEB: run \`mise run console_web_build\` first." >&2
  exit 1
fi
rm -rf site/public/play
mkdir -p site/public
cp -R "$CONSOLE_WEB" site/public/play
cd site
npm ci
npm run build
