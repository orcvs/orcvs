# 04 — Interpret terminal Play Functions into Play Commands

**What to build:** Interpret each valid terminal Play Function into one ordered Play Command in the Tick Plan, without producing a Cell result or moving musical timing decisions into the Playback Engine.

**Blocked by:** 03 — Commit atomic Cell results through Tick Plans.

**Resolution:** implemented

- [x] A root Play Function with a hexadecimal channel `0`–`F`, velocity `00`–`7F`, and valid MIDI note emits exactly one Play Command.
- [x] A Play Function never writes a Cell result; the existing placeholder zero output is removed.
- [x] A Play Function nested where another Function requires a value is diagnosed and emits no Play Command.
- [x] Velocity `80`–`FF` is diagnosed rather than clamped, and no Play Command is emitted.
- [x] Velocity `00` is preserved as an explicit MIDI zero-velocity Note On instruction.
- [x] Play Commands retain Expression order, and identical commands from consecutive Ticks remain present for exact repeated dispatch.
- [x] Interpretation tests assert Tick Plan outcomes without requiring MIDI hardware.

## Comments

**2026-08-26 — implemented (agent)**

Interpreter outcomes now distinguish Cell results from terminal Play Commands. Source collects valid root Play Commands in Expression order, carries evaluation failures into range-addressed Tick Plan diagnostics, and never creates a Cell write for Play. Tests cover the full velocity boundary, nested use, ordering, and repeated Tick dispatch without an output adapter.

**2026-08-26 — placeholder made observable again, arity pinned (agent)**

A code review found that `play_impl` had become a silent no-op: its only observable behaviour (`info!("Play: c: {}, v: {}, n: {}")`) was commented out when the function moved, so `>>` popped three arguments and returned `Number(0)` with no effect, leaving `c`/`v`/`n` as unused-variable warnings.

This ticket still owns the real work — emit one ordered Play Command and remove the placeholder Cell result. Only the regression was fixed: the log is restored, the `Number(0)` placeholder is unchanged, and a `TODO(issue 04)` in `lang/src/functions/mod.rs` points here.

The three-argument arity contract is now pinned by `functions::test::test_play_consumes_exactly_three_arguments` and `test_play_requires_three_arguments`, so the change this ticket makes to Play's output will be visible rather than silent. Both are characterization tests over pre-existing behaviour, mutation-checked to confirm they genuinely fail when the third `try_pop` is removed.

Note for this ticket's velocity-range criterion: operand parsing now rejects a leading sign, so `+0` is a type error rather than `0` (`lang::str_to_num`).
