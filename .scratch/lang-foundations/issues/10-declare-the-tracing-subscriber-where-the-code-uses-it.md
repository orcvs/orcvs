# 10 — Declare the tracing subscriber where the code uses it

**What to build:** Keep the console's tracing subscriber out of the builds that never compile the code which uses it.

**Blocked by:** None — can start immediately.

**Status:** resolved

**Tags:** Improvement

The console declares `tracing-subscriber` in its plain dependency table. The only code that uses it is the startup helper, which a non-WASM gate covers. So the WASM build carries the subscriber and the crates beneath it for code that target never compiles. Cargo's answer is the target-specific dependency table, which the crate already uses for one other dependency.

- [ ] The subscriber is declared only for the targets that compile the code which uses it.
- [ ] The subscriber and the crates it brings with it are absent from the console's WASM dependency graph, and still present for the native build.
- [ ] The crate that names the formatting subscriber type declares the feature that gates it, rather than relying on another feature to imply it.
- [ ] The native, WASM, and dependency-audit gates pass, and the lockfile is unchanged.

## Comments

This ticket was first written on a false premise and is recorded here rather than quietly corrected.

It originally claimed that the console received the subscriber's `fmt` feature by unification through `lang`, and that moving `lang`'s dependency would break the native console build. That is wrong. `tracing-subscriber` defines `ansi` as `["fmt", "nu-ansi-term"]`, and the console already declared `ansi`, so it always supplied `fmt` from its own manifest. Removing the explicitly added `fmt` line and forcing a rebuild proves it: the feature stays enabled through `ansi` and the startup code still compiles.

The error came from a misread of an inverted feature tree. That tree listed two enablers of `fmt` — `lang`, and the console's own `ansi` — and only the first was taken in. In an inverted tree those siblings are alternatives, so the output stated the correct answer and was read past.

Two things follow. Issue 05 never depended on this ticket, and its `Blocked by` line has been corrected. And the explicit `fmt` declaration is kept, because a crate that names `FmtSubscriber` should declare the feature gating it rather than rely on `ansi` to imply it — but it is kept as a deliberate choice, not as the fix for a break that could not happen.

What survived the correction is the ticket's title. Declaring a dependency where the code uses it is a real problem here; the WASM dependency graph was the place it actually lived.
