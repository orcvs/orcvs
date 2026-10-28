# 02 — Put the network behind a seam

Status: resolved
Type: task
Blocked by: 01

**What to build:** network output sends through one trait that takes a Network Send and the
destination, with a native socket adapter, a web refusal adapter and a recording adapter behind
`test-output-adapter` ([spec](../spec.md)).

- [x] The OSC encoder is a pure function the native adapter calls; the web adapter never encodes.
- [x] No `cfg` chooses behaviour inside the send itself; each target supplies its adapter.
- [x] Engine tests that bind loopback sockets record Network Sends instead.
- [x] `orcvs/tests/network_loopback.rs` still passes.
