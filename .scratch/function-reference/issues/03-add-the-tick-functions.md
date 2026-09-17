# 03 — Add the Tick Functions

**What to build:** The reference gains a Tick group: Clock, Delay, Euclidean, Increment, Interpolation and Random, each with an example whose result row is what the first Tick writes, so playing it shows the value changing.

**Blocked by:** 01 — Open on a Function reference, starting with the Arithmetic Functions.

**Status:** ready-for-agent

- [ ] One example each for `~.`, `~*`, `~%`, `~+`, `~>`, `~?`.
- [ ] Each result row is written as the first Tick writes it.
- [ ] Increment and Interpolation read their own result Cell; their examples start from a written value that makes the change visible.
- [ ] The one-Tick test covers the group. Later Ticks are expected to differ and are not asserted.
