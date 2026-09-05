# 11 — Persist the interpreter property's counterexamples

**What to build:** A failing interpreter property writes a regression file that reproduces it, as `docs/tooling.md` says every property does.

**Blocked by:** None — can start immediately.

**Status:** in-review

- [x] A failure in the interpreter property persists a counterexample.
- [x] The property keeps counting the cases it reaches, which is why it drives the runner directly.
- [x] The regression path is committed rather than ignored, matching the other two properties.

## Comments

The property drives the test runner directly instead of through the macro, deliberately and for a good reason: it asserts on how many generated Expressions the parser actually accepted, so that a generator producing only rejected input cannot pass while testing nothing.

The macro is also what supplies the source file the persistence layer needs to name a regression file. Driving the runner directly leaves that unset, and the persistence layer then declines to write, printing a notice rather than failing. So the property runs in both tiers and its counterexamples cannot be reproduced.

Naming the source file explicitly keeps both properties of the arrangement — the case counting and the persistence. The other two properties use the macro and already persist correctly, and no regression directory is ignored, so the rest of the promise in `docs/tooling.md` holds.

## Resolution

The `Config` the property builds now names its own source file:

```rust
let config = Config {
    source_file: Some(file!()),
    ..Config::default()
};
```

That is the value `proptest!` would have supplied, and it is all the persistence layer needed to name
`lang/proptest-regressions/interpreter.txt`. The direct-runner arrangement and both case-count
assertions are untouched, so the property still refuses to pass while testing nothing.

Verified by forcing a failure rather than by reading the API: inverting the `prop_assert!` produced

```
proptest: Saving this and future failures in .../lang/proptest-regressions/interpreter.txt
```

and a real `cc <seed>` line with a shrunk counterexample. The break was reverted and the file
deleted; it is a counterexample for a fake failure and does not belong in the repository.

`git check-ignore` answers 1 for the path, so no ignore rule excludes it. The contract's loop over
regression paths now names it alongside `lang/proptest-regressions/parser.txt` and
`orcvs/proptest-regressions/grid.txt`, so a later ignore rule fails the gate.
