# 09 — Send the network destination through the playback mailbox

Status: needs-triage
Type: task

**Background:** The console hands the playback engine the network destination (the host and port
UDP and OSC output sends to) on its own newest-wins `watch` channel
(`orcvs/src/playback.rs`, the `network_destination` sender), which the engine reads at each Tick
that sends. Every other request from the console, stopping, tempo and a MIDI destination change,
goes through the playback mailbox ADR 0056 describes (`orcvs/src/playback/mailbox.rs`).
[ADR 0112](../../../docs/adr/0112-the-network-destination-is-a-value-the-engine-reads-at-each-send.md)
records the separate channel as built; it stays as the record of today's code until this lands.

**What to do:**

1. Carry a network destination change as a mailbox request, alongside the existing ones, and remove
   the separate `watch` channel.
2. Keep what a user sees unchanged: a new destination applies from the next Tick that sends, and an
   unchanged one costs nothing per Tick.
3. Write the ADR that supersedes ADR 0112's channel decision, and update PLAY-350 and every marker
   in the same change.
4. Keep `orcvs/tests/network_loopback.rs` passing.
