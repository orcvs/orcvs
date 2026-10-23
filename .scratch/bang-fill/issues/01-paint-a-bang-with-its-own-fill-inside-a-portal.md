# 01 — Paint a Bang with its own fill inside a Portal

Status: resolved

**What to build:** `style::fill_overlay` answers no Portal for a Bang, as it does for a bound Function, so a Bang inside an Input or Output Portal keeps `source.bang`, `source.bang.background` and its own border. Locked still paints a Bang Function's spelling. ADR 0053's 2026-10-22 amendment records the rule.

**Blocked by:** None — can start immediately.

- [x] A Bang inside an Output Portal paints `source.bang` over `source.bang.background`, untinted under both built-ins.
- [x] A locked Bang still takes `locked.*`.
- [x] The style and Paint tests that pinned the Portal tint on a Bang are flipped.
- [x] The contrast and colour-vision gates pass for every shipped Theme.
