# 03 — Bind Portal inputs at Turn for Increment and Interpolation

**What to build:** At Turn, the Producer resolves each declared input Portal, reads the spelling
from working Source without per-Turn allocation, decodes it as the declared type, and supplies it
with cell operands to the Evaluator. Increment and Interpolation evaluate end-to-end through
Portal input binding rather than `TickInputs.previous`.

**Blocked by:** 02 — Declare Portal inputs on Increment and Interpolation.

**Status:** resolved

- [x] Turn resolves the ordinary result Portal for Increment and Interpolation and reads its
      spelling from working Source at the Producer's Turn.
- [x] Empty Portal Cells initialize as Number `00`; invalid or non-Number Portal content
      diagnoses rather than converting implicitly.
- [x] Increment returns `(previous + step) % modulus` with u64 widening before narrowing; zero
      modulus diagnoses under a message naming the Function.
- [x] Interpolation moves toward target without overshoot; rate `00` holds the current value.
- [x] Both Functions remain scalar exceptions to Sequence broadcasting.
- [x] Cross-Tick state is visible only in Source Snapshot Cells; live editing and stale-tail
      behaviour remain deterministic.
- [x] Orcvs Tick-by-Tick tests from `tick-functions/03` continue to pass against the new binding
      path.

## Comments

Turn execution consults the Function's Portal declaration and borrows its complete spelling
from working Source. Empty Cells decode as `00`; unavailable or invalid Portals diagnose after
cell operands. The six existing feedback Source tests pass through the new path. A new carried
Portal test proves both Functions read an earlier Turn's write instead of Snapshot Cells.
The `source_execute_tick_portal_inputs` benchmark makes the feedback path available to CI;
no local timing comparison was run.
