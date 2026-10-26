# 12 — Publish the site build with releases

**What to build:** A release publishes the web console it names, and the site it deploys. The
`console-web` workflow keeps each console build as the `console-web` artifact, with relative URLs
so it runs from any path, and the `site` workflow keeps each site build as `site-<sha>` and deploys
only that. A release attaches the console, as an archive, beside the native builds the
`/download/` route lists once releases exist, and the site it deploys takes that same console.
Artifacts expire, so the release copies the build rather than linking to the run. See `../spec.md`
(Build, Hosting) and ticket 05.

**Blocked by:** 05, and a decision on how native releases are built and tagged.

**Status:** needs-triage

- [ ] A tagged release carries the web console archive built from its commit, and the site its
      tag deploys serves that console.
