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

## The requirements restate these decisions

[`docs/requirements/`](../requirements/README.md) restates the accepted ADRs in current words, one
rule in one place, as a snapshot at the revision each document names. The ADRs stay binding: a
requirement that disagrees with an accepted ADR is a defect in the requirement. Accepting an ADR
updates the owning requirement and its snapshot marker in the same change, as the requirements
index describes; an ADR that changes nothing a requirement states says so in its pull request.

The topic that restates each ADR, superseded ADRs included so a reader can see where their
surviving clauses went:

| Requirement | ADRs |
| --- | --- |
| [Language](../requirements/language.md) | 0005, 0007, 0008, 0010, 0011, 0013, 0015, 0017, 0018, 0019, 0021, 0023, 0024, 0025, 0026, 0029, 0030, 0033, 0035, 0039, 0043, 0049, 0054, 0061, 0062, 0063, 0067, 0069, 0070, 0071, 0072, 0073, 0074, 0075, 0103 (spelling and building), 0104, 0105, 0107, 0108, 0109 |
| [Evaluation](../requirements/evaluation.md) | 0003, 0004, 0006, 0009, 0012, 0014, 0020, 0028, 0031, 0032, 0034, 0036, 0057, 0060, 0065, 0066, 0068, 0077 |
| [Playback and outputs](../requirements/playback.md) | 0001, 0002, 0016, 0037, 0041, 0055, 0056, 0059, 0076, 0085, 0087 (the synth as a destination), 0088, 0090, 0099, 0103 (sounded commands) |
| [Console](../requirements/console.md) | 0022, 0038, 0040, 0042, 0044, 0045, 0046, 0047, 0048, 0050, 0051, 0052, 0053, 0058, 0078, 0079, 0080, 0081, 0082, 0083, 0084, 0089, 0092, 0095, 0096, 0098, 0100, 0102 |
| [Site](../requirements/site.md) | 0086, 0087 (page messages and timing), 0091, 0093, 0094, 0097, 0101 |

### Words the older ADRs use

ADRs keep the vocabulary they were written in. Where it has changed:

| Older ADRs say | Read as | Changed by |
| --- | --- | --- |
| Sequence, Sequence value, broadcasting over a Sequence | No such value: a List is Cells in a claim, read by Track and Read | ADR 0063, ADR 0067 |
| Overview | Minimap | ADR 0098 |
| Pending Function, pending state | Invalid input: a Function with an unwritten required operand does not evaluate | ADR 0069 |
| Rest, a Play that rests | A Play whose note is not written sends nothing | ADR 0108 |
| Address Function, the `&` directional addressing family | The Read, Write and Copy families | ADR 0070 |
| Margin of the Source View | No margin; Pan stops at the Grid's edge | ADR 0084 |
