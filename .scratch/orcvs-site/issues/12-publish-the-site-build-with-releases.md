# 12 — Publish the site build with releases

**What to build:** A release publishes the same site build that deploys, so the web console a
release names is the one its users ran. The `site` workflow already keeps each build as the
`site-<sha>` artifact and deploys only that artifact; a release attaches it, as an archive, beside
the native builds the `/download/` route lists once releases exist. Artifacts expire, so the
release copies the build rather than linking to the run. See `../spec.md` (Build, Hosting) and
ticket 05.

**Blocked by:** 05, and a decision on how native releases are built and tagged.

**Status:** needs-triage

- [ ] A tagged release carries the site archive built from its commit, and that archive is the one
      the tag's `site` run deployed.
