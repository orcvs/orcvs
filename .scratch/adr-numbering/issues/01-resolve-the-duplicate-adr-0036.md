# 01 — Resolve the duplicate ADR 0036

**What to fix:** `docs/adr/` holds two accepted ADRs numbered 0036 —
`0036-pulse-functions-refuse-a-sequence-operand.md` and
`0036-reserve-result-cells-before-their-width-exists.md`. They are different decisions about
different mechanisms, both concerning Sequences, and the repository cites both as "ADR 0036" with no
filename. A reader chasing a citation has no way to tell which one they want, and the subject matter
gives no signal that they opened the wrong file.

**Status:** needs-triage

- [ ] One of the two ends is chosen and recorded here with its reason: renumber one ADR, or disambiguate the citations.
- [ ] No citation anywhere resolves to two decisions.
- [ ] Whatever is chosen, `docs/adr/` cannot grow a third collision without it being noticed.

## Verification

`cargo fmt --all -- --check`, `cargo clippy --package <crate> --all-targets --locked -- -D warnings`,
and `PROPTEST_CASES=32 cargo nextest run --package <crate> --locked` for whichever crates' comments
are touched. If `.scratch/` changes: `node --test scripts/tests/roadmap.test.ts` and
`node scripts/roadmap.ts > /dev/null`.

## Comments

Measured on `main` at `2ffdd4c`: 81 occurrences of the string `ADR 0036` outside `target/`. 63 are in
Rust sources and `CONTEXT.md`, 15 in `.scratch/`, and 3 in `docs/adr/` itself.

The 3 inside `docs/adr/` are not part of the problem, and they are the evidence for what the fix
should look like. ADR-to-ADR citations already use a linked form that carries the filename —
`[ADR 0036](0036-pulse-functions-refuse-a-sequence-operand.md)` in ADR 0012, and
`[ADR 0036](0036-reserve-result-cells-before-their-width-exists.md)` twice in ADR 0032. Both name
exactly one decision. The convention exists; it simply cannot be used from a Rust comment.

`lang/src/atom.rs` is where the cost is plainest. Sixteen citations, meaning both ADRs, interleaved:

- `:413` and `:419` mean the pulse ADR — ":419 Declared by ADR 0036's Delay `~*` and Euclidean `~%`".
- `:445` means the reserve ADR — "Tick scheduling reads this, per ADR 0036, to decide how many Cells one [result occupies]".

Those are 26 lines apart in one declaration block. `:1746` and `:1774` mean the pulse ADR again;
`:1794`, `:1803` and `:1821` mean the reserve one, within thirty lines of each other.
`orcvs/src/source/tick.rs` carries 24 citations and `lang/src/functions/tick.rs` 8.

Two ends to fix, and this issue exists to choose one rather than to start typing.

**Renumber one ADR.** The highest number in `docs/adr/` is 0038, so the pulse ADR becomes 0039 — or
the reserve one does. This fixes the root cause: every bare citation becomes unambiguous without
being touched, because only one ADR answers to the number. The cost is that an ADR number appears in
commit messages, pull requests and issue files that cannot be rewritten, so the renumbered ADR needs
a line recording its former number, and anyone reading history has to know to look for it. It is
also worth establishing whether renumbering an accepted ADR is acceptable practice here at all;
nothing in `docs/adr/` states a policy either way, and there is no ADR index or README.

**Disambiguate the citations.** Leave both numbers and write "ADR 0036 (reserve result Cells)" or
"ADR 0036 (pulse Functions)" at each of the 78 sites outside `docs/adr/`. This touches a lot of prose
and each site has to be read to know which ADR it means — a mechanical pass, but not an automatic
one. It also leaves the collision in place for the next citation written.

Whichever is chosen, the third checkbox is the one that matters beyond this fix: nothing today would
notice a fourth ADR taking a number already used. A check over `docs/adr/` filenames is cheap and
belongs wherever the tooling contract is enforced — `scripts/check-tooling-contract.sh` is the
existing home for that kind of assertion.

A related but separate defect, already filed: `.scratch/lang-foundations/issues/09` records the
broken intra-doc link at `lang/src/atom.rs:821`, which makes
`RUSTDOCFLAGS="-D warnings" cargo doc -p lang` fail. That gate is where a documentation convention
would otherwise be enforced, and it is red today.

Found by a three-reviewer pass on `function-replacement-04-05`, which reported it against two new
test comments citing ADR 0036 bare. Those two are not the defect; they follow a convention 78 other
sites already set.
