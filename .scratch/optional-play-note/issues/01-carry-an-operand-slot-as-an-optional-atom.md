# 01: Carry an operand slot as an optional Atom from the Parser to binding

**What to build:** An operand slot holds an optional Atom from the Parser's Positioned Entry, through the Language Map's resolved operands and the Tick's operand decode, to `bind_atom`. An absent value means the slot is blank or a nested Function answered a Clear. This is a prefactor: nothing a performer sees changes. See `../spec.md`, "The slot carries an optional Atom".

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Each module on the path passes an absent operand on without interpreting it; only binding branches on it.
- [ ] A plain operand refuses an absent value with the diagnostic it gives today: the same message and anchor for a blank slot, and `nested computation at column X, row Y cleared its operand` for a nested Clear.
- [ ] A blank plain operand is still diagnosed in the Language Map before any Tick.
- [ ] Every existing test passes unchanged, apart from mechanical signature updates.
- [ ] `evaluate` tests in lang show a plain operand refusing an absent value with today's message.
