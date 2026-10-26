#!/usr/bin/env bash
# Builds the site as it deploys, from the repository root: Trunk writes the release console into
# `site/public/play/` with `$SITE_BASE/play/` as its base, and Astro builds with `SITE_BASE` as
# its own and copies `public/` into `site/dist/` untouched. The last line fails the build if the
# two base paths disagree, which would break every asset URL the console loads. `SITE_BASE` is
# empty: the site is served from its domain's root. `mise run site_build` runs this, locally and
# in the `site` workflow's build job.
set -euo pipefail
SITE_BASE="${SITE_BASE:-}"
export SITE_BASE

cd console
env -u NO_COLOR trunk build --release --locked --public-url "$SITE_BASE/play/" --dist ../site/public/play
cd ../site
npm ci
npm run build
grep -F "<base href=\"$SITE_BASE/play/\"" dist/play/index.html > /dev/null
