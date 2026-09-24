# 07 — Open a Source File

**What to build:** File > Open… (⌘O) on native picks a `.orcvs` file with a native dialog and opens it as the environment, through the same Open `file-new/01` names.

**Blocked by:** 05 — Read and write a Source File; 06 — Track the open Source File.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] `rfd` is added through `rust-dependency-change`, synchronous API, native targets only, with its rationale recorded; `mise run audit_deps` passes.
- [ ] The dialog filters to `.orcvs` and still allows any file.
- [ ] Unsaved changes are guarded as `06` specifies before the dialog opens.
- [ ] A refused file leaves the running Source, path and unsaved state untouched and shows the refusal as a notice.
- [ ] A read file becomes the environment; its path becomes the open file and the Source counts as saved.
- [ ] Open does not appear on web, and ⌘O does nothing there.
- [ ] Tests cover open, refusal and a cancelled dialog without driving a real dialog.
- [ ] Scoped gates for `console` pass, and `mise run check_wasm` still compiles.
