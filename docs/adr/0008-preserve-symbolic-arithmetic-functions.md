# Put the behaviour family first in symbolic Function names

Orcvs groups Functions by behaviour. A symbolic Function name puts its behaviour-family glyph first and its operation glyph second. Thus, related Functions share a visible prefix, and a reader can identify the evaluation family before the specific operation.

The initial family prefixes are `.` for numeric behaviour, `~` for Tick and feedback behaviour, `*` for activation, `&` for addresses, `@` for Source read and write, `:` for Sequence structure, and `!` for terminal output. A second glyph selects the operation or direction within that family. The initial Address Functions are the four directional Jump Functions `&^`, `&v`, `&<`, and `&>`. More complex Cell-address forms remain deferred. Reversing an earlier suffix example does not make that spelling canonical.

The Terminal Output family contains every Function that sends an outbound effect and returns no language value. It covers MIDI, UDP, OSC, and application commands. This family boundary does not settle individual Function suffixes, message value types, or operand contracts.

The MIDI output Functions use `!>` for Raw Play, `!~` for Timed Play, `!%` for Monophonic Play, `!c` for Control Change, and `!b` for Pitch Bend. The `~` identifies the Tick lifetime of Timed Play. Monophonic Play preserves Orca's recognizable `%` glyph and owns one voice per output adapter and MIDI channel. Control Change accepts `!c channel controller value`; Pitch Bend accepts `!b channel lsb msb`, with the least-significant data byte first to match MIDI wire order — together the two bytes form MIDI's 14-bit pitch-bend value, though Orcvs continues to expose them as two direct 7-bit bytes rather than one combined operand. Both accept direct hexadecimal MIDI bytes without scaling or clamping, require channel `00`–`0F`, and require every MIDI data-byte operand to lie in the 7-bit range `00`–`7F`. Invalid operands diagnose and emit no MIDI output.

The remaining Terminal Output Functions use `!u` for UDP, `!o` for OSC, and `!$` for Application Command. UDP and OSC remain deferred until Orcvs has a suitable message value. Application Command retains Orca's recognizable `$` glyph and sends a command only to the Orcvs host application's explicit command dispatcher. It never invokes an operating-system shell or arbitrary executable. Its command value encoding remains deferred until Orcvs has a suitable text or message value.

The initial Sequence Functions use operation glyphs that show their behaviour: `:-` makes a Range, `:<` reverses order, `:&` concatenates two Sequences, `:?` selects one Atom, and `:=` replaces one Atom.

The Source Functions reserve `@<` for Source Read and `@>` for Source Write. The arrow shows whether a value enters the Expression from Source or leaves the Expression for Source. Source Read may return one Atom or one intact Sequence, and Source Write accepts one Atom or one intact Sequence through a Portal; their concrete operands remain non-parseable until the Cell-address form is decided.

The Numeric Functions use `.+` for Addition, `.-` for Subtraction, `.|` for Absolute Difference, `.x` for Multiplication, `./` for Division, `.%` for Modulo, `.<` for Minimum, `.>` for Maximum, and `.=` for Equality. The same family uses `.v` for conversion to Number and `.^` for checked conversion to Note under ADR 0021. Multiplication uses the ASCII `x` because it remains legible beside the Bang Atom `**`.

The Time Functions use `~.` for Clock, `~*` for Delay, `~+` for Increment, `~?` for Random, `~%` for Euclidean, and `~>` for Interpolation. An operation glyph can recur in another family because the prefix identifies the behaviour first. Thus, `~+` and `~%` remain distinct from arithmetic Addition `.+` and Modulo `.%`.

Function names continue to use standard ASCII. Unicode remains deferred because each Cell currently contains one ASCII byte.
