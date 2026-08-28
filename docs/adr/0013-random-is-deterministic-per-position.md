# Random is deterministic per Position

The Random Function `?~ seed minimum maximum` selects inclusively between normalized bounds, so reversed bounds describe the same range and equal bounds return that Number. It derives each result from the explicit seed, absolute Tick, Function Position, and Pattern index rather than activation history. Identical Source Snapshots interpreted at the same Tick therefore produce identical Tick Plans; skipped activations skip those Tick samples, Functions at different Positions have independent reproducible streams, and moving one intentionally changes its stream.

Random follows ordinary Pattern broadcasting. Pattern bounds produce element-wise results, and Pattern index participates in each element's deterministic identity so equal bounds in different positions do not accidentally share a stream.
