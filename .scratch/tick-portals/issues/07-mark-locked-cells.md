# 07: Mark the Cells a Halt locked

**What to build:** The Render Frame marks the two Cells of the Function of each
Expression root the last Tick's Halts locked with a Locked role, and the console
paints them in a style of their own, as Orca colours the Cells an operator
locks. See `../spec.md` section 6.

**Blocked by:** 04, 05

**Status:** resolved

- [x] A lock with a `Locked` verdict marks the locked root's Function Cells,
      where the displayed revision still begins an Expression with a Function
      there; every other verdict marks nothing.
- [x] A retained lock goes out until the next Tick when an edit reaches the
      Halt or the locked root's own spelling, and an edit elsewhere, the
      locked root's operands included, leaves it marked.
- [x] Output Portal paints over Input Portal over Locked, as Orca draws a port
      over a locked Cell; Locked paints a bound Function and a Bang, and a
      locked Function's spelling a Function also reads or writes keeps its own
      paint.
- [x] `locked.*` Theme channels in both built-ins, measured by the contrast
      checks over the facts a lock can mark.
- [x] The derivation still allocates nothing for a record that holds a lock.
