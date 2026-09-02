# Portals resolve Tick Plan destinations

A Portal is one internal Cell destination resolved while interpreting a Source Snapshot, not a language value or persistent state. A Portal may resolve any Cell; future Cell-addressing models, including an infinite canvas, can therefore change destination resolution without changing Function evaluation or Tick Plan commit semantics.

An ordinary result sends one Atom or intact Sequence through one Portal. A Source Function may resolve multiple Portals as one effect bundle. Tick planning validates the complete bundle before admitting any write, then expands its encodings and clears in emission order. A clear writes a space to Source; space does not become an Atom or Sequence member. An empty Sequence plans no writes. An ordinary result writes only its current encoding and never clears a stale tail outside that Footprint. Every admitted write participates Cell-wise in ADR 0020's producer and emission order.
