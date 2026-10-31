# PROTOTYPE: 303-style channel — throwaway, not for merge

Question: does a mono, sliding, accented saw (or square) through a resonant 4-pole lowpass with its
own decay envelope sound like a 303, next to channel `02` as it is today?

Run: `cargo run --release --manifest-path synth/prototype/acid/Cargo.toml -- <out dir>` writes
`acid-prototype.wav` and `channel-02-today.wav`: the same sixteen-step line at 130 BPM for eight
bars. In the acid render, cutoff rises and falls over the eight bars, env mod rises, resonance
goes up from bar 5 and the waveform turns square from bar 7, each set per step the way a Source
sweeps a Control Change every Tick. Ticket: `.scratch/acid-channel/issues/01`.
