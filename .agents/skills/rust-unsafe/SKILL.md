---
name: rust-unsafe
description: Implement or review Rust involving unsafe blocks, unsafe functions or traits, FFI, raw pointers, memory layout, pinning, atomics, or aliasing in this repository. Combine with rust-change for edits and rust-review for independent review.
---

# Rust unsafe

1. Identify the exact safety boundary, safe callers, and invariants required before and after the operation.
2. Prefer a safe implementation; otherwise minimize the unsafe scope and expose a safe wrapper.
3. Put a `SAFETY:` comment immediately above each unsafe block, proving every required invariant.
4. Test invalid boundaries and invariant-preserving behaviour; add target, property, or concurrency tests where the risk requires them.
5. Run the scoped gate plus Miri on supported focused tests. For FFI or layout, also check each supported target and external contract.
6. Report the safety argument, commands run, unsupported checks, and residual risk.
