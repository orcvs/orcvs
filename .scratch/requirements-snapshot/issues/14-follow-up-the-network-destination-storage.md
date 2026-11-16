# 14 — Follow up where the network destination is stored

**Status:** needs-triage

**Background:**
[ADR 0112](../../../docs/adr/0112-the-network-destination-is-a-value-the-engine-reads-at-each-send.md)
was kept as one record on 2026-11-15, so its amendment of ADR 0085 stands: the console stores the network
destination itself, under the eframe Storage key `orcvs_network_destination`
(`console/src/network.rs`), natively and only with the `persistence` feature. `config.toml` never
holds it. Three loose ends follow from that choice.

**What to do:**

1. The web build reads `orcvs_network_destination` at startup when `persistence` is on, but nothing
   on the web writes it, and a browser cannot send UDP or OSC (PLAY-360). Decide whether the read
   should compile out on wasm32, as `store` already does, or stay as harmless.
2. `.scratch/udp-osc-output/spec.md` line 48 and ticket
   `.scratch/udp-osc-output/issues/04-set-the-network-destination.md` line 8 still say the
   destination is saved "with the other settings". Add a dated note to each pointing at ADR 0112;
   the spec stays as history.
3. Without `persistence`, or on a fresh profile, the destination starts at the default ports, and a
   reader cannot set it from `config.toml`. Decide whether a `config.toml` default is wanted; if it
   is, that takes an ADR, since ADR 0112 records the console as the only writer and reader.

**Done when:** each item is ruled or done, and `docs/requirements/playback.md` and
`docs/requirements/console.md` still match the code once ADR 0112 is accepted.
