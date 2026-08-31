# 02 — Recover the Source lock from poisoning

**What to fix:** `SourceCommander` unwraps every `RwLock` guard. One panic inside a `Source` method
poisons the lock. Every later read and write then panics, and the editor dies.

**Status:** ready-for-agent

- [ ] A panic inside one `Source` method does not stop later reads and writes.
- [ ] `SourceCommander` and `PlaybackEngine` treat lock poisoning the same way.
- [ ] A test poisons the lock, then asserts that a later read still returns.

## Comments

`orcvs/src/source/mod.rs`, lines 30, 38, 42, 52, 56, 63, and 71. Each one calls `.read().unwrap()`
or `.write().unwrap()`.

`orcvs/src/playback.rs:18` already holds the opposite policy for its own `Mutex`:

```rust
fn lock_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
```

The two modules disagree, and `SourceCommander` holds the more dangerous of the two locks.

Panics inside `Source` are reachable by design, because the type uses assertions for its invariants.
`Grid::assert_owns` panics. `plan_tick` asserts that an interpreter result is ASCII. `set_source`
indexes a slice.

The cascade matters because of who holds the lock. The Playback clock task calls
`SourceCommander::execute`, which takes the write guard. A panic in that background task poisons the
lock. The next Render Frame calls `read_revision_cells`, which unwraps a poisoned read guard and
panics on the UI thread. One bad Tick therefore takes down the whole console, not one Tick.

Write a `lock_recover` equivalent for `RwLock`. There is no ready-made one, because `RwLock` has two
guard types. Give it two functions, or one generic over the guard.

Recovery is correct here. A poisoned `Source` still holds a valid `String` and a valid `Grid`,
because every panic path either happens before a mutation or leaves the buffer whole.
