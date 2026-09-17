# 04 — Add the Jump, Bang and movement Functions

**What to build:** The reference gains groups for the Jumps, the Directional Bang Functions, the Self-Banging Functions and Halt, each laid out so that playing it acts only inside its own example area and never overwrites another example.

**Blocked by:** 01 — Open on a Function reference, starting with the Arithmetic Functions.

**Status:** ready-for-agent

- [ ] Jumps: `&^`, `&v`, `&<`, `&>`, each with a value at its input to copy.
- [ ] Directional Bangs: `*^`, `*v`, `*<`, `*>`, each with a Bang source (for example an Equality that holds) so it emits when played.
- [ ] Self-Banging: `^^`, `vv`, `<<`, `>>`, each with a path that ends inside its own example area.
- [ ] Halt: `*!`, with a Bang source and a root directly south for it to lock.
- [ ] A test ticks the reference repeatedly (enough Ticks for every mover to stop) and asserts no Cell outside each example's own area changes.
- [ ] The one-Tick exact-result test excludes these example areas, since movement is their result.
