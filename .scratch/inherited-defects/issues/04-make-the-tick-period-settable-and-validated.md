# 04 — Make the Tick period settable and validated

**What to fix:** `Bpm` holds a private `usize` with no constructor and no setter. The tempo is fixed
at `Bpm(20)`. `delay_ms` divides by that field, so a zero value would panic.

**Status:** needs-triage
**Implementation:** complete

- [x] The user can change the tempo.
- [x] `Bpm` refuses a zero value at construction, as `MarkerSpacing` does.
- [x] `delay_ms` cannot divide by zero.
- [x] A test asserts that `Bpm::new(0)` returns `None`.

## Comments

`orcvs/src/opts.rs:32`, `:60-65`, and `:70`.

```rust
pub struct Bpm(usize);

impl Bpm {
    pub fn delay_ms(&self) -> u64 {
        let ms = (60000 / self.0) / 4;
        ms as u64
    }
}
```

`Opts::new` writes `Bpm(20)`. That is the only construction in the workspace. The field is private
and there is no `new`, so no code outside `opts.rs` can build a different tempo. The console
therefore plays at one fixed tempo, and the user cannot change it.

Two neighbours in the same file show the intended shape. `MarkerSpacing` and `HighlightSpacing` both
wrap a `NonZeroUsize` and both offer a fallible `new`. `Bpm` wraps a bare `usize` and offers
nothing.

The division is safe today only because the one construction is a literal. It becomes a panic the
moment a tempo control exists and passes zero. The same zero also reaches
`PlaybackEngine::start`, which returns `PlaybackStartError::ZeroTickPeriod`.

Triage is needed because this is two changes, not one. Validation is small and can land alone.
A tempo control is a user-interface decision: where the control lives, what range it accepts, and
whether a tempo change restarts playback or takes effect on the next Tick.

Land the validation first. Give `Bpm` a `new(usize) -> Option<Self>` over `NonZeroUsize`, and keep
`delay_ms` as it is. The division is then safe by construction.
