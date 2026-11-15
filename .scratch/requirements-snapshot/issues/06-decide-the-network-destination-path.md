# 06 — Decide how the network destination reaches the Playback Engine

**Status:** needs-triage

**What to decide:** the network destination reaches the Playback Engine outside ADR 0056's request
slots. `docs/requirements/playback.md` lists this under "Open decisions".

**Done when:** an ADR states whether the network destination goes through the request slots, and the
requirement or the code follows it.

**Progress, 2026-11-15:** proposed as
[ADR 0112](../../../docs/adr/0112-the-network-destination-is-a-value-the-engine-reads-at-each-send.md),
which records the built watch channel and amends ADR 0085's storage clause to the eframe Storage key
`orcvs_network_destination`. Waiting on Toby to accept or change it, including whether to amend ADR
0085 or change the code.
