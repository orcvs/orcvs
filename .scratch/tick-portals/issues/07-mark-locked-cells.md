# 07: Mark the Cells a Halt locked

**What to build:** The Render Frame marks the two Cells of the Function of each
Expression root the last Tick's Halts locked with a Locked role, and the console
paints them in a style of their own, as Orca colours the Cells an operator
locks. See `../spec.md` section 6.

**Blocked by:** 04, 05

**Status:** resolved

- [x] A lock with a `Locked` verdict marks the locked root's Function Cells;
      every other verdict marks nothing.
- [x] A retained lock follows the `Declared` rule: an edit to the Halt puts it
      out until the next Tick, and an edit elsewhere, the locked root
      included, leaves it marked.
- [x] Output Portal paints over Input Portal over Locked, as Orca draws a port
      over a locked Cell; Locked paints a bound Function and a Bang.
- [x] `locked.*` Theme channels in both built-ins, measured by the contrast
      checks over the facts a lock can mark.
- [x] The derivation still allocates nothing for a record that holds a lock.
