# 03 — Send OSC from `!o`

Status: ready-for-agent
Type: task
Blocked by: 02

**What to build:** An activated `!o` sends its Payload as one OSC message through the network output
adapter from ticket 02 ([spec](../spec.md), ADR 0085).

- [ ] Evaluation splits the Payload into words and validates the address, the optional `,` type
      string and each argument, emitting one command of typed arguments: int32, float32 or string.
- [ ] Guessing without a type string: one to eight uppercase hexadecimal digits are an int32
      whose bits they spell (`40` is 64, `1B8` is 440, `440` is 1088, `FFFFFFFF` is -1), a decimal matching `-?[0-9]+\.[0-9]+` is a float32, anything else is a
      string. `C4` is the int32 196; `ab`, `.5`, `5.` and `1e3` are strings.
- [ ] A bad address, a letter other than `i`, `f` or `s`, a count mismatch or a misfit word
      diagnoses, names the word, and sends nothing.
- [ ] The adapter encodes OSC 1.0 by hand (padded address, padded type tags, big-endian arguments)
      and sends it to the OSC destination. No bundle or timetag.
- [ ] Encoder tests compare bytes against the OSC 1.0 spec's own examples, and a loopback test
      receives a message.
