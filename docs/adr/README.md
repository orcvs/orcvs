# Architecture decision records

One decision per file, named `NNNN-<slug>.md`, numbered from `0001`. The repository cites ADRs by
bare number in Rust comments and in `CONTEXT.md`, where a link cannot be written, so a number has to
name exactly one decision for a citation to be followable.

## Numbering

Take the next number above the highest file in this directory. A gap is not a free number: `0027` is
missing here because a branch that has not landed claims it, and reusing it would land the collision
this directory has already had once.

## When two ADRs collide on a number

**The ADR that was still on a branch is the one that gets renumbered. The ADR already on `main`
keeps its number.** Two branches open at once pick the same next number and only the second to land
creates the collision, so this is the rule whoever lands second applies — without asking, and
without waiting for a reader chasing a citation to find it.

Renumbering an accepted ADR is therefore expected practice here. The renumbered file records its
former number beside its Status line, because commits, pull requests and `.scratch/` issue files
written before the rename cite the old number and cannot be rewritten. ADR 0039 is the worked
example: it was accepted as ADR 0036, which
[ADR 0036](0036-reserve-result-cells-before-their-width-exists.md) had already taken on `main`.

`scripts/check-tooling-contract.sh` fails when two files here share a leading number, so a third
collision cannot land unnoticed the way the second one did — `0036` named two accepted decisions for
a day, and left 78 bare citations outside this directory resolving to either of them.
