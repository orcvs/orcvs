# 11 — Record the network-failure diagnostic class

**Status:** needs-triage

**What to decide:** ADR 0055 names the classes a full diagnostics log evicts by (PLAY-120). The code
adds a fifth, `NetworkFailure` (`orcvs/src/playback/diagnostics.rs` `Class`), ranked just above
Overrun and counted in Omitted Diagnostics (decisions D9). `docs/requirements/playback.md` lists it
under "Accepted but not built, or built differently".

**Done when:** an ADR amendment to 0055 names the class and its rank, or the code follows ADR 0055's
classes.
