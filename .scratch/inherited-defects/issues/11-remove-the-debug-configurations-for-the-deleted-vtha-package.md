# 11 — Remove the debug configurations for the deleted vtha package

**What to fix:** `.vscode/launch.json` holds two configurations for a package called `vtha` and a
benchmark called `parser_benchmark`. Neither exists.

**Status:** ready-for-agent
**Implementation:** complete

- [x] Every debug configuration names a package in the workspace.
- [x] Every debug configuration names a target that the package builds.
- [x] The tooling contract covers the package names.

## Comments

`.vscode/launch.json:51-60` debugs library tests in `--package=vtha`. Lines 70-79 debug
`--bench=parser_benchmark --package=vtha`.

`Cargo.toml:2` lists the members: `["lang", "orcvs", "shell"]`. There is no `vtha`. Both
configurations fail with `package ID specification 'vtha' did not match any packages`.

The benchmark cannot be restored as written. `scripts/check-tooling-contract.sh` asserts that no
manifest declares `criterion`, so a `parser_benchmark` bench target is against the current contract.

Delete both configurations. Add one for `lang` if you want to debug the parser, because `lang` is
the package that now holds it.

The crate-split branch corrected the other two configurations in the same file, which named the
renamed `console` package, and added contract assertions for them. Extend those assertions to cover
whatever configurations remain.
