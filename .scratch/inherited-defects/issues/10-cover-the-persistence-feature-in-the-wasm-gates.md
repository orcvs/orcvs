# 10 — Cover the persistence feature in the WASM gates

**What to fix:** `check_wasm` lints with `--features persistence`, then builds without it.
`test_wasm` also runs without it. A fault that only appears with the feature passes both gates.

**Status:** ready-for-agent

- [ ] The WASM build and the browser tests agree with the WASM lint about features.
- [ ] A persistence fault in the WASM build fails a gate.
- [ ] The tooling contract asserts the chosen feature set.

## Comments

`mise.toml:92-102`:

```
[tasks.check_wasm]
run = '''
cargo clippy --workspace --lib --target wasm32-unknown-unknown --features persistence --locked -- -D warnings
cd shell
env -u NO_COLOR trunk build --locked
'''

[tasks.test_wasm]
run = 'wasm-pack test --headless --firefox shell --test wasm --locked'
```

The lint on line 95 passes `--features persistence`. The build on line 97 and the browser tests on
line 102 do not. The gate therefore lints one configuration and builds a different one.

`main` has the same split, with `--package console` in place of `--workspace`, so the crate split
did not cause it.

CLAUDE.md states the rule that this breaks: "Exercise explicit feature combinations; do not assume
every feature composes." The WASM target is where the rule matters most, because `serde`, `web-time`
and `gloo-timers` all behave differently there.

Decide which configuration the WASM gate defends, then make all three commands use it. If the
released web application enables persistence, pass the feature to Trunk and to `wasm-pack`. If it
does not, drop the feature from the lint, and add a second lint for the released configuration.

Add the matching assertions to `scripts/check-tooling-contract.sh`. That script already guards the
`--locked` flags on both commands, so the feature flags belong beside them.
