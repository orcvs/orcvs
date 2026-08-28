# Use two-Cell hexadecimal Numbers

Orcvs represents every Number as one unsigned byte encoded by two uppercase hexadecimal Cells (`00`–`FF`). Orca uses base 36 because one glyph must carry each value; Orcvs deliberately spends two Cells to gain byte alignment, familiar tracker notation, and a natural fit with MIDI data. Orca's base-36 range and modulo-36 arithmetic are therefore design inputs rather than compatibility requirements.
