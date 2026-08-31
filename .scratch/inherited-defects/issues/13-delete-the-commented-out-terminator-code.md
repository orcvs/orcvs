# 13 — Delete the commented-out terminator code

**What to fix:** `glyph.rs` holds a commented-out `is_terminator` function and a commented-out test
for it. Both sit beside the live `is_terminator_bytes` that replaced them.

**Status:** ready-for-agent
**Implementation:** complete

- [x] The commented-out function and its test are gone.
- [x] `is_terminator_bytes` and its test are unchanged.

## Comments

`orcvs/src/glyph.rs:92-99` holds the commented function. Lines 160-175 hold the commented test.
`is_terminator_bytes` at line 101 and its test at line 178 are the live pair.

Dead code in comments states that work is pending. Here it is not. The `&str` version was replaced
by the byte version, and git holds the old text if anyone wants it.

One commented arm inside the block reads `"+" => true`, which suggests that a Marker was once a
terminator. `is_terminator_bytes` does not treat it as one. If that difference is a real open
question, write it as an issue with a title. Do not leave it as a comment.
