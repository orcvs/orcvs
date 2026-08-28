# Random is deterministic per Position

The random Function accepts an explicit seed, minimum, and maximum and derives its result from that seed, the current Tick, and the Function's Position. Identical Source Snapshots interpreted at the same Tick therefore produce identical Tick Plans, while otherwise identical random Functions at different Positions have independent reproducible streams and moving one intentionally changes its stream.
