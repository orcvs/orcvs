# 01 — Run the rustdoc gates on pull requests

**What to build:** A rustdoc warning fails the pull-request tier. The same two `cargo doc`
invocations the merge tier runs today — default features and persistence — run there instead, the
tooling contract pins them there, and the records stop calling rustdoc merge-only.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] The pull-request tier runs both rustdoc invocations with warnings denied.
- [x] The merge-only tasks no longer run rustdoc.
- [x] The tooling contract pins the new home of each invocation and no longer pins them on the merge
      tasks.
- [x] `docs/tooling.md` and the agent contract name rustdoc as a pull-request gate; what remains
      merge-only is the browser suite and persistence at full case count.
