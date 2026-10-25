# 02 — Make the splash logo

Status: ready-for-agent
Blocked by: 01

**What to build:** `logo-splash.png` in the console's assets, derived from `orcvs.png` as
[`../spec.md`](../spec.md) describes, so the native splash and the web loading screen draw the
same logo.

- [ ] The logo is white with the artwork's grey level as alpha, 512 px square, and under 100 KB.
- [ ] Composited on black, it differs from `orcvs.png` resized to 512 px (Lanczos) by a mean of at
      most 3 and a maximum of at most 24 of 255.
- [ ] It is made by this command, run once, not a build step. The source artwork is not committed.

```sh
convert orcvs.png -colorspace Gray -resize 512x512 -level 2%,100% -posterize 32 \
  -alpha copy -fill white -colorize 100 -define png:compression-level=9 logo-splash.png
```

`-level 2%,100%` clears the artwork's near-black noise floor to full transparency.
