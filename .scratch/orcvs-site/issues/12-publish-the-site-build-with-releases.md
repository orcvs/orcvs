# 12 — Publish the site build with releases

**What to build:** A release publishes the web console it names, and the site it deploys. The
`console-web` workflow already publishes each console built on `main` as `console-web.tar.gz` on
the rolling `console-web` release, with relative URLs so it runs from any path, and Cloudflare's
site build downloads it from there. A tagged release attaches the same archive beside the native
builds the `/download/` route lists once releases exist, and the site deployed for the tag serves
that console. See `../spec.md`
(Build, Hosting) and ticket 05.

**Blocked by:** 05, and a decision on how native releases are built and tagged.

**Status:** needs-triage

- [ ] A tagged release carries the web console archive built from its commit, and the site its
      tag deploys serves that console.
