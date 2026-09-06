# Original Orca Bang behavior and the implication for issue 06

Investigated 2026-09-06. Primary sources pinned to JavaScript Orca commit
`223e45c84db14cb77fb1d31e29a3ad94652080d8` and Orca-c commit
`9df9786e2ad3c01955cdf4cdd5ae1fffad8fa5cc`. No Orcvs language decisions are changed here.

## Finding

Original Orca does **not** make a Bang spatially inert merely because another operator uses its
Cell as an input. Neighbor activation and permission to execute are different questions:

- Activation checks whether any cardinal neighbor contains the character `*`. It does not check
  whether that neighbor is locked or belongs to another operator's input.
- Locking prevents an operator at that Cell from executing. Therefore a locked `*` does not run
  its erase behavior, while it remains visible to neighboring operators' activation checks.
- An unlocked `*` erases when its own turn is reached in row-major execution. Whether another
  operator sees it depends on order and on intervening writes.

These rules are explicit in JavaScript's
[neighbor check](https://github.com/hundredrabbits/Orca/blob/223e45c84db14cb77fb1d31e29a3ad94652080d8/desktop/sources/scripts/core/operator.js#L92-L98),
[operation loop](https://github.com/hundredrabbits/Orca/blob/223e45c84db14cb77fb1d31e29a3ad94652080d8/desktop/sources/scripts/core/orca.js#L60-L81),
[port locking](https://github.com/hundredrabbits/Orca/blob/223e45c84db14cb77fb1d31e29a3ad94652080d8/desktop/sources/scripts/core/operator.js#L41-L49),
and [Bang implementation](https://github.com/hundredrabbits/Orca/blob/223e45c84db14cb77fb1d31e29a3ad94652080d8/desktop/sources/scripts/core/library.js#L488-L499).
The C implementation independently agrees: [neighbor check](https://github.com/hundredrabbits/Orca-c/blob/9df9786e2ad3c01955cdf4cdd5ae1fffad8fa5cc/sim.c#L46-L59),
[locking MIDI inputs and Bang erasure](https://github.com/hundredrabbits/Orca-c/blob/9df9786e2ad3c01955cdf4cdd5ae1fffad8fa5cc/sim.c#L269-L279),
and [row-major execution with lock/sleep exclusion](https://github.com/hundredrabbits/Orca-c/blob/9df9786e2ad3c01955cdf4cdd5ae1fffad8fa5cc/sim.c#L748-L786).

## Minimal example

Original Orca uses one character per operator, `:` for MIDI, and `.` for an empty Cell. These are
Orca examples, not Orcvs Source syntax.

```text
:04C*1......
....:04Cf1..
............
```

The first `:` owns the five input Cells to its right: channel `0`, octave `4`, note `C`, velocity
`*`, duration `1`. It has no neighboring Bang, so it sends no note. Its input Cells are nevertheless
locked. The second `:` has `*` immediately above it, so it sends one note. The locked `*` survives
this Tick. This is the original behavior, not an accidental consequence of Orcvs parsing.

The JavaScript [MIDI implementation](https://github.com/hundredrabbits/Orca/blob/223e45c84db14cb77fb1d31e29a3ad94652080d8/desktop/sources/scripts/core/library.js#L542-L579)
checks activation within its operation; the general run method locks ports even if that operation
returns without sending a note. In C, MIDI locks ports before its activation check. The tested
observable result is identical.

## Executed probes

Both pinned implementations produced these results from one Tick:

| Case | MIDI messages | `*` after Tick |
| --- | ---: | --- |
| Example above: Bang in another MIDI operator's velocity input | 1 | Present |
| Replace that velocity `*` with `f` | 0 | Absent initially |
| Standalone `*` directly above MIDI | 0 | Erased |
| Standalone `*` directly below MIDI | 1 | Erased |
| `:*4Cf1......`: Bang in first MIDI input | 1 | Present |

The C probe also tested a string payload: `;a*b........` above `..:04Cf1....` sends one MIDI note
and preserves the `*` in the UDP operator's locked payload. Its
[string scan locks payload Cells](https://github.com/hundredrabbits/Orca-c/blob/9df9786e2ad3c01955cdf4cdd5ae1fffad8fa5cc/sim.c#L323-L339)
without removing their ability to activate a neighbor. No JavaScript/C difference was found in the
five shared cases; this is not a claim of complete equivalence between the implementations.
The JavaScript probe also repeats the locked velocity case for a second Tick: another MIDI message
is sent and the `*` still survives. A locked input Bang is not intrinsically a one-Tick pulse.

Local reproduction artifacts:

- `/tmp/orca-bang-reference`: pinned JavaScript checkout.
- `/tmp/orca-bang-probe.cjs`: JavaScript probe, run with `node /tmp/orca-bang-probe.cjs`.
- `/tmp/orca-c-bang-reference`: pinned C checkout.
- `/tmp/orca-c-bang-probe.c`: C probe.
- C compile command: `cc -std=c99 -I /tmp/orca-c-bang-reference /tmp/orca-c-bang-probe.c /tmp/orca-c-bang-reference/sim.c /tmp/orca-c-bang-reference/gbuffer.c /tmp/orca-c-bang-reference/vmio.c -o /tmp/orca-c-bang-probe`.
- C run command: `/tmp/orca-c-bang-probe`.

## Implication for Orcvs

Issue 06's acceptance criterion that an operand Bang must activate no root is **not justified by
alignment with original Orca**. Original Orca demonstrates precisely the opposite. Its no-expiry
behavior for a locked input is real, but comes from execution locking, not from making the Bang
invisible to spatial activation. Treat activation and expiry independently when revisiting the
issue.

The earlier Orcvs example `!>00**C4` also does not establish a valid parsed Bang operand: the
current parser reports `expected a number, found "**"`. It demonstrates a Bang spelling inside
malformed Expression Source. Original Orca has character inputs, so it does not directly answer
what Orcvs's typed malformed Expressions should do.

There is also an explicit existing divergence: Orcvs [ADR 0006](../../docs/adr/0006-bang-activation-includes-self-banging-functions.md)
reads Bangs from the Source Snapshot and removes them at commit, whereas original Orca reads and
erases during its mutable row-major pass. Thus original Orca's above/below asymmetry cannot be
copied simply by changing how Language Units are classified. Strict alignment of timing requires
reviewing that recorded Orcvs decision.

Architectural inference: the Language Map can gain depth and locality by owning answers about
Expression membership and spatial meaning, but a single blanket "operand means no spatial
meaning" rule conflates two independently observable behaviors. Deepening this module should
concentrate the agreed language rules behind its interface, not introduce a speculative seam or
adapter before deciding those rules. Tests should exercise activation and survival separately
through the same interface callers use.


## Manual entry is permitted in original Orca

Original Orca does not require every `*` to originate as Function output. Its
[keyboard handler](https://github.com/hundredrabbits/Orca/blob/223e45c84db14cb77fb1d31e29a3ad94652080d8/desktop/sources/scripts/commander.js#L156-L160)
passes the typed character to the
[cursor write operation](https://github.com/hundredrabbits/Orca/blob/223e45c84db14cb77fb1d31e29a3ad94652080d8/desktop/sources/scripts/cursor.js#L83-L89),
which accepts `*` as an allowed character. `node /tmp/orca-manual-bang-probe.cjs` passed through
those actual upstream handlers and asserted that the Cell contains `*` without evaluating any
producing Function. A Bang can nevertheless be modeled as a transient value rather than a callable
Function in Orcvs; that design question is distinct from permitting manual entry.
