# 05 — Widen the Function kind to value or effect

**What to build:** Make `FunctionKind` distinguish a Function that answers a value from one that answers an effect, rather than a Function that answers a value from one that performs Terminal Output.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

**Sources of truth:** ADR 0028 states the value-or-effect rule; ADR 0029 states that the kind must carry it and that spelling must not decide it; ADR 0024 states that spelling does not decide what an Atom is; ADR 0025 makes `Sequence::new` the single membership point.

- [ ] `FunctionKind` distinguishes value from effect.
- [ ] `Function::is_terminal` either keeps its narrower meaning honestly or is replaced by the question each caller actually asks.
- [ ] The Interpreter's nesting guard asks whether a Function answers a value.
- [ ] Tick planning's activation gate asks the question it means, not a narrower one that happens to coincide.
- [ ] `Sequence::new` refuses a Function Atom by its declared kind, per ADR 0029.
- [ ] No check enumerates spellings, and no check infers a kind from a family prefix.
- [ ] A Function added with an effect kind joins every one of those callers by its definition alone.

## Comments

`lang/src/atom.rs` declares `enum FunctionKind { Value, Terminal }` and `is_terminal` asks `matches!(self.kind(), FunctionKind::Terminal)`. ADR 0029 states the problem exactly: value-versus-effect and value-versus-Terminal-Output "coincide today only because Terminal is the one effect kind `FunctionKind` defines". Raw Play is currently the only effect Function that exists, so nothing is wrong yet and everything will be wrong at once.

Three callers ask the narrow question today and mean the wide one. `lang/src/interpreter.rs:53` rejects a terminal Function anywhere but the root. `orcvs/src/source/tick.rs:338` gates activation on `is_terminal_root`, and its own comment says it asks "the Function's own classification rather than naming `!>`" so that it "keeps this in step with the canonical Function definitions as the family grows" — which is true of the mechanism and false of the question. `Sequence::new` will be the third.

This blocks more than it looks. `spatial-tick-planning/03` and `05` each carry an acceptance criterion requiring membership to refuse a Function "by its declared kind, per ADR 0029" — 03 for the Directional Bang Functions, 05 for Halt. Neither ticket owns the kind change, so today it lands as a side effect of whichever runs first, in whatever shape that ticket needs. Doing it here once, ahead of both, is the difference between one declaration and two partial ones.

Note what this ticket does not decide. ADR 0029 is explicit that a Directional Bang Function and the Self-Banging Function it emits are refused membership by different mechanisms — `^^`, `vv`, `<<`, and `>>` are their own `Atom` variant and stay one arm of an exhaustive match, while `*^` is an ordinary Function Atom refused by its kind. Collapsing the two would obscure the activation asymmetry that is the reason both forms exist.
