# 04 — Latch native network failures

Status: resolved
Type: task
Blocked by: 03

**What to build:** a failed native network send is reported once until it changes, and the latched
failure is the Playback observation's "latest send failed" field ([spec](../spec.md), ADR 0090).

- [x] A successful send, a new network destination and the start of a run clear the latch.
- [x] The console's network status clears when the observation's failure does.
- [x] The web build still diagnoses every activation.
- [x] The doc comment that says network failures are never latched is gone.
