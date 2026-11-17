# 12 — Decide what happens to a message that is late while audio starts

**Status:** needs-triage

**What to decide:** ADR 0087 says a message later than the synth's delay sounds at once (SITE-043).
`synth/web/synth-host.js` drops messages that arrived while the audio context was starting and are
more than 500 ms past their planned instant (decisions D9). The drop is described only in the
resolved `orcvs-site/16`. `docs/requirements/site.md` lists it under "Accepted but not built, or
built differently".

**Done when:** an ADR amendment to 0087 records the drop, or `synth-host.js` sounds those messages
at once.
