# Time and feedback are explicit Tick inputs

Clock, Delay, and Euclidean Functions adapt Orca's established formulas using the current Tick as an interpretation input, hexadecimal Numbers, and first-class one-Tick Bang results. Clock `.~ rate modulus` returns `floor(Tick / rate) % modulus`: rate is the number of Ticks per step, modulus is the number of steps, and their product is the complete cycle length. Delay `*~ rate modulus` Bangs exactly when that Clock cycle wraps, including when modulus is `01`; Orca's special case that Bangs every frame for modulus `1` is not retained because it breaks the shared cycle model. A zero rate or modulus diagnoses.

Euclidean `:~ hits steps` retains Orca's bucket distribution and phase. Zero hits produces no Bangs, equal hits and steps Bangs every Tick, zero steps diagnoses, and hits greater than steps diagnoses.

Increment `+~ step modulus` and interpolation `>~ rate target` read their previous visible Number through the ordinary result Portal in the current Source Snapshot, treating an empty Portal as the initial value `00` and diagnosing a present non-Number. Increment wraps by its nonzero modulus and diagnoses modulus `00`. Interpolation approaches its target by at most rate without overshooting; rate `00` intentionally holds the current value. Cross-Tick behaviour therefore remains Source-visible rather than hidden Interpreter state.
