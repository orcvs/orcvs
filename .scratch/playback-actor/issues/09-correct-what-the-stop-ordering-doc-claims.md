# 09 — Correct what the stop-ordering doc claims

**What to build:** Make the documented ordering between the published stop and the report that
explains it match the code, in both directions or in neither.

`PlaybackInner::stop()` publishes `Stopped` at `orcvs/src/playback.rs:674` and only then calls
`send_safety_reset()` at `:676`, which can record an output failure. The test helper
`diagnostics_once_stopped` documents the opposite as an invariant at `:2977-2979`: "The engine records
the report before it publishes the stop that report explains, so a drain after this answers is a
drain that cannot have missed it."

That sentence is true of the `Drop` path only, where `:834` reports and `:837` stops. It is false of
the ordinary stop, where the publish comes first and a failing `safety_reset` records after it.

Nothing fails today, because the helper's two callers are the panic and unexpected-termination tests
and both go through `Drop`. The hazard is the next test written against the helper's stated rule: it
will wait for `Stopped`, drain, and miss a safety-reset failure that had not been recorded yet, and
it will do so intermittently.

Either the helper's doc says what is actually guaranteed — "on the `Drop` path" — or `stop()`
publishes after the safety action so the stated rule is true everywhere. The second has a cost worth
weighing: the published state is what the console gates Space on, and moving the publish behind a
device call delays it by the length of that call.

**Status:** resolved

- [ ] The doc at `orcvs/src/playback.rs:2977-2979` and the code agree.
- [ ] If the fix is in the doc, it names the `Drop` path as the scope of the guarantee and says what
      a caller after an ordinary stop may still be waiting for.
- [ ] If the fix is in `stop()`, the delay the publish now sits behind is stated, because the
      published state is what gates the console's Space handling.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package orcvs --all-targets --locked -- -D
warnings` and `PROPTEST_CASES=32 cargo nextest run --package orcvs --locked`.

## Comments

Filed from the `playback-actor` review ledger as **CR-10**, minor.

Minor because nothing is red and nothing is user-visible. Filed anyway because the wrong half of the
sentence is the half a later author will build on: a test-synchronisation helper is consulted for its
stated rule, not read against the function it names.

### Resolved

Fixed in the doc rather than in `stop()`. `diagnostics_once_stopped` now names
the `Drop` path as the scope of its guarantee, says that the ordinary
`PlaybackInner::stop` publishes before it attempts the safety action, and tells
the next author what to do instead on that path — collect until the expected
diagnostics arrive, the way
`a_backend_that_panics_being_silenced_does_not_abort_the_process` does.

The ordering in `stop()` is left as it is, and the reason is now written down:
the published state is what the console gates Space on, so moving the publish
behind a device call would delay it by the length of that call.
