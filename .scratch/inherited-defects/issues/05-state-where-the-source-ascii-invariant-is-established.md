# 05 — State where the Source ASCII invariant is established

**What to fix:** The SAFETY comment on `Source::set_source` says that the byte "is validated as
ASCII". It does not say where. One of the two callers validates several steps earlier, in a
different function. A reader cannot check the claim from the comment.

**Status:** ready-for-agent

- [ ] The SAFETY comment names the two callers and the check each one makes.
- [ ] The comment names `plan_tick`'s assertion as the guard for the Tick path.
- [ ] The `unsafe` block keeps its present size.
- [ ] No behaviour changes.

## Comments

This issue is smaller than a review of the crate split reported. That review called the write
undefined behaviour. It is not. The invariant holds. Only the comment is wrong.

`orcvs/src/source/model.rs:435-444`:

```rust
/// Writes one already-validated ASCII byte at `idx` without
/// recalculating Expressions.
fn set_source(&mut self, idx: usize, byte: u8) {
    // SAFETY: `byte` is validated as ASCII and `idx` is bounds-checked
    // before any mutation, so the String stays valid UTF-8
```

`set_source` has two callers.

`edit` at line 194 is reached from `Source::set`, which calls `check_content` at line 226. That
function accepts one byte in `0x20..=0x7e` and rejects everything else. The comment describes this
path correctly.

`commit_tick` at line 399 is the other caller. It writes `write.content as u8`, and `CellWrite`
holds a `char` at line 53. That cast truncates. The guard is not in `commit_tick`. It is in
`plan_tick` at line 353:

```rust
assert!(
    encoded.is_ascii(),
    "Interpreter Cell results must preserve the Source ASCII invariant"
);
```

`plan_tick` builds every `CellWrite` from `encoded`, so no non-ASCII `char` can reach a plan.
`commit_tick` is private, and `execute` at line 310 is the only path to it. `execute` always calls
`plan_tick` first. No caller outside the module can pass a plan of its own, even though `TickPlan`
and `CellWrite` are public.

The invariant therefore holds on both paths. The defect is that a reader must find an assertion in
another function to prove it, and the comment does not point there.

One related line deserves a note in the same comment. Line 379 writes
`for (offset, content) in encoded.chars().enumerate()` and then indexes the byte buffer with
`target_idx + offset`. That mixes a character count with a byte offset. It is correct only because
of the same ASCII assertion. Say so, so that a later change to the assertion is understood to affect
both lines.
