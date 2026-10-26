# UDP and OSC send the Cells up to their closer

Status: accepted. Settled in `.scratch/udp-osc-output/`. Amends [ADR 0008](0008-preserve-symbolic-arithmetic-functions.md), [ADR 0016](0016-use-distinct-raw-and-timed-play-functions.md) and [ADR 0019](0019-map-orca-capabilities-onto-orcvs-language-families.md), which deferred `!u` and `!o` until Orcvs had a message value. [ADR 0063](0063-a-list-is-cells-in-a-claim-not-a-value.md) retired the value model that deferral waited on, so no message value is coming. The research is `docs/research/udp-osc-output.md`.

Orca's `;` sends the glyphs east of it, up to the first `.`, as one UDP datagram. Its `=` sends an OSC message to `/` plus one glyph, with every later glyph as an int32 of its base-36 value. Orcvs keeps the idea, which is that the Cells after the Function are the message, and changes the details where Orcvs Source differs from Orca's.

## Decision

**A Payload is the Cells from the end of a `!u` or `!o` spelling through the next `!` on its row, which closes it.** `!uhello!` sends `hello`. With no closer, the Payload runs to the end of the row, as an unclosed Comment does ([ADR 0074](0074-a-comment-closes-at-the-next-introducer.md)). The Parser finds the closer by search, so the Payload's extent is fixed when the Source is parsed and the schedule stays fixed. A Payload cannot contain `!`, and there are no escapes. Its Cells are never parsed.

**A Payload's Cells are ordinary Cells, and its Function reads them at its Turn.** The occupied Cells are the spelling and the closer, as a Comment's are its two delimiters. A Write writes into the Payload, and a Read reads from it. The Function reads its Payload after every writer of those Cells has taken its Turn, so a Write lands in the same Tick. A write that changes or removes the closer changes the extent from the next parse, as an overwritten Comment closer does.

**Each Cell sends its character, and an empty Cell sends a space.** `!u` sends exactly those bytes as one UDP datagram, with no terminator. An empty Payload, `!u!`, sends nothing.

**An OSC Payload is an address, then an optional type string, then arguments, separated by spaces.** The first word is the address and must start with `/`. A next word that starts with `,` is an OSC type string, such as `,ifs`, with one letter per argument. Without one, each argument's type is guessed from its shape:

- Two hexadecimal digits are an int32 holding that Number, so `40` sends 64, as everywhere else in the Grid.
- A decimal with a point, such as `0.5` or `440.0`, is a float32.
- Any other word is a string.

A type string accepts only `i`, `f` and `s`. Under it, an `i` word must be two hexadecimal digits, an `f` word must be a decimal with a point, and an `s` word is sent as written. An address that does not start with `/`, any other type letter, a type string whose length differs from the argument count, or a word that does not fit its type each diagnose and send nothing, as an out-of-range MIDI operand does. A two-Cell pair has no Note type outside an operand (`CONTEXT.md`, Operand Literal), so `C4` sends the int32 196.

**One activation sends one plain message, in Tick Plan order, when its Tick runs.** No OSC bundle and no timetag is sent. OSC 1.1 leaves a timetag's meaning to each receiver, and many receivers act on arrival anyway. `!u` and `!o` follow every other Terminal Output Function: they perform only when activated, are invalid where a value is required, and write no Cell result.

**The destination is a console setting, never Source.** One host and two ports, saved with the console's other settings, default to `127.0.0.1`, UDP port `49161` and OSC port `49162`, which are Orca's.

**Sends go through the Playback Engine to their own network output adapter, alongside MIDI.** A send that fails reports a diagnostic and never stops the performance ([ADR 0065](0065-an-error-never-stops-the-performance.md)). UDP and OSC hold no state on the receiver that Orcvs knows how to reset, so stopping sends nothing on them, and the MIDI safety action does not reach them.

**The web build parses and paints `!u` and `!o` exactly as the native build does.** A browser cannot open a UDP socket, so an activated `!u` or `!o` there diagnoses that the build cannot send UDP or OSC. A Source file therefore means the same thing in both builds.

**Orcvs only sends.** It does not receive UDP or OSC. Orca's UDP input drives its own commander, which belongs with Application Command `!$` and stays deferred.

## Considered options

- **Quotes, `!u"hello"`.** `"` was kept free for text by ADR 0070. Quotes cost two Cells to the closer's one, and `!` closes the Payload with the glyph that opens it, as `||` closes a Comment.
- **The first empty Cell, as Orca's `.` does.** A Payload could not hold a space, so an OSC message would need another word separator. The end would also move whenever the blank after a Payload filled.
- **A length operand, as Orca-c's `=` does.** Every edit to the text would need the count changed too.
- **One-glyph OSC addresses, as Orca does.** They cannot spell the addresses that SuperCollider or TouchDesigner expect.
- **Every argument a string.** That is the simplest option, but receivers dispatch on the OSC type, and a numeric control given the string `"40"` either errors or needs converting at the receiver.
- **Every argument an int32, as Orca does.** There would be no strings and no floats.
- **A `"` Text unit that may stand anywhere in the Grid.** Nothing needs free-standing text, so the Payload exists only after a Function that takes one.

## Consequences

- `!` now has two jobs on a row: it opens every Terminal Output Function and closes a Payload. In `!uhi!c` the Parser closes the Payload at the second `!`, while a reader may see Control Change `!c`. A blank Cell after the closer keeps a row readable.
- The OSC type rules read words as text. A float has no Orcvs value spelling, so a float can come only from a Payload, written or put there by a Write.
- The v1 definition of done keeps UDP and OSC as accepted deferrals. This decision settles their design, not their release.
