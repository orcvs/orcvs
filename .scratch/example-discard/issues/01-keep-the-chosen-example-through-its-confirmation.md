# 01 — Keep the chosen example through its confirmation

Status: ready-for-agent

**What to build:** `Help → Examples` keeps the chosen example through the discard confirmation, so `EXAMPLES` is the one catalogue examples are opened from.

Today `DiscardConfirmation.discard` is a `fn(&mut Console)`, which cannot carry the choice, so `EXAMPLE_CONFIRMATIONS` repeats each catalogue position as a closure, one per `EXAMPLES` entry in the same order (`console/src/console/files.rs:435`), and the menu zips names to actions (`console/src/console/menu_bar.rs:368`). Nothing but convention keeps the two in order.

- `DiscardConfirmation` becomes an enum of discard actions with `question()` and `discard(&mut Console)` methods. One variant is `Example(&'static Example)`. The native-only variants (Function Reference, Open, Quit) and the native and web question texts stay `cfg`-gated as they are.
- `load_example` takes `&Example` instead of an index.
- `EXAMPLE_CONFIRMATIONS` and the zip go; the menu iterates `EXAMPLES`.
- The confirmation stays `Copy` and allocation-free.

Preserve ADR 0086's storage rules for the Linked and small consoles, ADR 0094's link names and ADR 0097's Copy Link, and the native and web differences in when a discard asks.

## Acceptance criteria

- [ ] For every example, a kittest writes the Source, picks the example, confirms Discard, and finds the chosen example open.
- [ ] For every example, Cancel keeps the written Source.
- [ ] The existing every-example opening test and the settings-preservation tests pass unchanged.
- [ ] No parallel per-example table remains.

## Comments

### 2026-11-03 — Opened

From the architecture review of HEAD `6b124537` (candidate 02), settled in a grilling session. Rejected: a boxed closure, and a fn pointer with an added example field. Land after `playback-start-policy/01`.
