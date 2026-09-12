# 04 — Retire the marker and highlight palette tokens

**What to fix:** `ConsolePalette` carries two colour tokens for Glyphs that no longer exist, and two
documents record them as part of the console's palette.

**Blocked by:** 03 — Delete the two Glyph variants nothing can produce.

**Status:** needs-triage

**Triage owes the same decision `03` states, plus one of its own.**
`typed-source-paint/01` already carries the acceptance line "`PALETTE.marker` and `PALETTE.highlight`
are deleted. `restyle-egui-console/02` names twenty-two tokens as the decided record; it goes to
twenty, and that issue records why." So the deletion is already owned. What is *not* owned anywhere
is `console/src/theme.md:15-16`, which records both tokens and which no ticket in any effort
mentions. If `typed-source-paint/01` is taken, this ticket reduces to that one document and the
count corrections, and should be re-scoped rather than closed.

- [ ] `ConsolePalette` (`console/src/style.rs:7-32`) no longer declares `marker` or `highlight`.
- [ ] `PALETTE` (`console/src/style.rs:34-58`) no longer assigns `rgba(46, 82, 72, 0.44)`
      (`:45`) or `#2A5A4E` (`:46`).
- [ ] `console/src/theme.md:15-16` no longer lists a Marker or a Highlight token.
- [ ] The palette token count is corrected wherever it is stated in words, not left stale.
- [ ] `restyle-egui-console/02` and `console-testing/03` are amended in the same change, or this
      ticket is held until they settle. See below.
- [ ] Every remaining token keeps its exact value. No colour shipped today changes.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings`,
      and `PROPTEST_CASES=32 cargo nextest run --package console --locked` pass, plus the two tracker
      gates for the `.scratch/` edits.

## Comments

**Why this is separate from `03` rather than folded into it.** `03` is forced: deleting the enum
variants breaks the exhaustive match at `console/src/style.rs:79-80`, so those two arms go in the
same compilation. The *fields* are not forced. `ConsolePalette` is a `pub struct` in a library
crate, so two `pub` fields nothing reads raise no `dead_code` warning and `03` compiles cleanly
leaving them behind. That makes this a real choice rather than fallout, and it has a blast radius
`03` does not.

**The blast radius is two other efforts' open tickets, both `ready-for-agent`.**

`console-testing/03` — "Pin the console palette to its recorded values" — has an acceptance line
requiring "every one of the twenty-two dark tokens" asserted at its exact value, naming marker and
highlight in the list, plus a second line requiring the `marker` token to be "asserted as a value
like the rest, with a comment recording that `restyle-egui-console/02` holds no capture to it".
Both become unsatisfiable once the tokens are gone.

`restyle-egui-console/02` — "Prototype-aligned console palette" — has an acceptance line requiring
"the 22 palette tokens hold exactly these values", listing `marker` and `highlight` among them, and
a further line requiring that "the `marker` token is recorded as a compile-time requirement, not a
visual one" because `console/src/style.rs:76` "must still map it" to keep the match exhaustive.
After `03` there is nothing to keep exhaustive, so that line describes a constraint that no longer
exists.

Both tickets reasoned correctly from the code in front of them: the token *was* a compile-time
requirement of an exhaustive match. `03` removes the requirement rather than contradicting the
reasoning. But two `ready-for-agent` tickets cannot be left naming tokens that were deleted, so
either they are amended in this change or this ticket waits for them.

**A pre-existing discrepancy the amender will meet, recorded so it is not mistaken for this
change's doing.** `ConsolePalette` declares twenty-three fields (`console/src/style.rs:9-31`) and
`console/src/theme.md:5-23` lists twenty-three tokens, including `comment` / "Comment: `#7A8784`".
`restyle-egui-console/02`, `console-testing/03` and `typed-source-paint/01` all say "twenty-two" and
omit `comment` from their enumerations. That is an existing off-by-one in those tickets, not
something this effort introduces; whoever amends them should write the number that is actually true
after the deletion — twenty-one — rather than subtracting two from a wrong number and arriving at
twenty.

**One line in `console/src/theme.md` must survive.** `:38-42` is the paragraph that explains the
seams "replace the historical `+` Marker Glyphs, leaving every empty Cell visually empty while
preserving the configured Marker spacing as geometry". That sentence is the clearest record in the
repository of *why* these tokens are being retired, and `05` cites it. Delete the two token lines at
`:15-16`; do not delete the explanation, though the phrase "configured Marker spacing" in it becomes
wrong the moment `05` lands and should be updated there, not here.

**Why `needs-triage` and not `ready-for-agent`.** The engineering is trivial — delete two fields, two
literals and two list entries. What is not trivial is that it invalidates written acceptance
criteria in three other efforts that a maintainer scoped, and that one of those efforts already
claims half the work. Whether to amend, to fold, or to hold is a sequencing call about someone
else's effort, and an agent should not make it unilaterally.
