# Use distinct raw and timed Play Functions

Orcvs provides raw Play as `>> channel velocity note` and timed Play as `>? channel velocity note length`. Both are terminal Functions activated by a Bang landing on their root or by a directional Always Function targeting them. Separate names keep each prefix Function's arity fixed and explicit rather than making length optional or overloading one spelling. Raw Play leaves Note Off under explicit Source control through velocity `00`; timed Play places its complete lifetime in the Tick Plan so the Playback Engine can schedule the corresponding Note Off without inferring musical intent.

The initial terminal output family also includes MIDI control change `cc` and pitch bend `pb`. OSC and UDP output remain deferred until Orcvs has message and text-like values rather than encoding them as exceptional raw Source reads.
