# 04 — Latch native network failures

Status: ready-for-agent
Type: task
Blocked by: 03

**What to build:** a failed native network send is reported once until it changes, and the latched
failure is the Playback observation's "latest send failed" field ([spec](../spec.md), ADR 0090).

- [ ] A successful send, a new network destination and the start of a run clear the latch.
- [ ] The console's network status clears when the observation's failure does.
- [ ] The web build still diagnoses every activation.
- [ ] The doc comment that says network failures are never latched is gone.
