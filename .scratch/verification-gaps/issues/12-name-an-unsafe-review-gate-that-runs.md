# 12 — Name an unsafe review gate the toolchain can run

**What to build:** The repository contract names a gate for unsafe, FFI, layout, raw-pointer, and atomic changes that can actually be run against the pinned toolchain.

**Blocked by:** None — can start immediately.

**Status:** in-review

- [x] The named gate exists as a task, or the contract stops naming it.
- [x] If the gate is kept, the toolchain declares the channel it needs and a task invokes it.
- [x] The decision is recorded in this issue.

## Comments

The gate is named twice in prose and nowhere else. It ships on nightly, and the toolchain is pinned to stable with two components, so an agent following the contract is asked for a tool the repository does not provide.

The workspace holds exactly one unsafe block — an in-place ASCII byte write with a stated safety invariant — which is precisely what the named gate would cover, and it has never been run under that gate.

The repository treats nightly as deliberately optional and non-blocking, which argues for rewording rather than adding a nightly job. Either resolution is defensible; naming a gate that cannot run is not, because it makes the contract unfollowable at the one place it matters most.

## Decision — the contract stops requiring Miri and names what already runs

No nightly job was added. `rust-toolchain.toml` pins stable with two components, and `AGENTS.md`
states that nightly is optional and non-blocking; adding a required gate that needs a channel the
repository deliberately does not pin would contradict the paragraph three sections above it.

The rewording names a gate that already runs on every clippy invocation. `Cargo.toml`'s
`[workspace.lints]` tables already carry `unsafe_op_in_unsafe_fn = "deny"` and
`undocumented_unsafe_blocks = "deny"`, so an unsafe block without a stated safety invariant fails
`check_pull_request`, `check_wasm`, and the scoped gate alike. The workspace's one unsafe block — the
in-place ASCII byte write in `orcvs/src/source/model.rs` — passes it today with a `// SAFETY:` comment
that names what is at stake and why every byte involved is single-byte ASCII.

Miri is still the tool this gate would prefer, and the reworded line says so: run it deliberately
under nightly rather than treating it as something a change owes. `AGENTS.md` and
`.agents/skills/rust-unsafe/SKILL.md` were changed together, since the skill repeated the same
requirement.

What this does not claim: the lints check that an invariant is written down, not that it is true. A
change to the byte write still needs the focused tests the reworded line also names, and a reviewer.
