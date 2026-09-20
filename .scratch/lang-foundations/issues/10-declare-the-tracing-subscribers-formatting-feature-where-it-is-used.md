# 10 — Declare the tracing subscriber's formatting feature where it is used

**What to build:** Make the console's own manifest request the tracing-subscriber feature its startup code needs, rather than receiving that feature by accident through another workspace crate's dependency graph.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

**Tags:** Improvement

The console builds a formatting subscriber at startup. The type it names is gated behind a tracing-subscriber feature the console's manifest does not request; the build succeeds only because `lang` requests that feature as an ordinary dependency and Cargo unions features across the graph. That coupling is invisible at both call sites and is the expand half of an expand–contract pair with issue 05, which removes the accidental provider.

- [ ] Every crate that names the formatting subscriber type requests the feature gating it in its own manifest.
- [ ] No crate depends on feature unification through another workspace crate to compile its own code.
- [ ] The native and WASM gates pass and the lockfile is unchanged.
