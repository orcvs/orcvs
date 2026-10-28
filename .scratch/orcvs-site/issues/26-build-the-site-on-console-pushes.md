# 26 — Build the site on pushes that change the console

**What to build:** Cloudflare's build of the `orcvs` Worker runs on every push to `main` that
changes the console, so ticket 25's wait can deploy it. Ticket 05 asked for build watch paths that
left the console's paths out, relying on the deploy hook to rebuild the site; ADR 0092 removes the
hook. #323's merge did start a site build, which suggests the watch paths were never restricted,
but this needs checking in the dashboard.

**Blocked by:** 25

**Status:** ready-for-human

In the Cloudflare dashboard: Workers & Pages → `orcvs` → Settings → Build → Build watch paths.
Leave them unset (build on every push), or add `console/*`, `orcvs/*`, `lang/*`, `synth/*`,
`scripts/stage-synth.sh`, `.cargo/config.toml`, `Cargo.toml`, `Cargo.lock`,
`rust-toolchain.toml` and `.github/workflows/console-web.yml` to the site's own paths.

- [ ] A push to `main` that changes only the console starts a production build of the site.
