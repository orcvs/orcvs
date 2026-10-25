# 02 — Make the splash logo

Status: ready-for-agent
Blocked by: 01

**What to build:** `logo-splash.png` in the console's assets, derived from `orcvs.png` as
[`../spec.md`](../spec.md) describes, so the native splash and the web loading screen draw the
same logo.

- [ ] The logo is white with the artwork's brightness as alpha, 512 px square, and under 100 KB.
- [ ] Composited on black it matches the original artwork; tinted black on white it reads as the
      same logo.
- [ ] The command that made it is recorded in the commit message. It is not a build step, and the
      source artwork is not committed.
