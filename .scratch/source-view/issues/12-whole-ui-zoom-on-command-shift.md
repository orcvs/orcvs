# 12: Whole-UI zoom on command-shift

**What to build:** The Source View took command `=`, `-` and `0` from egui's whole-UI zoom. Command-shift `=` and command-shift `-` enlarge and shrink the whole UI — the menus, the Panel and the Diagnostics — so a viewer who relied on it keeps it. See ADR 0045.

**Blocked by:** 04

**Status:** ready-for-agent

- [ ] Command-shift `=` enlarges the whole UI, and command-shift `-` shrinks it, within egui's own zoom limits.
- [ ] Neither chord changes the Source View's Zoom, and neither writes to the Source.
- [ ] Command `=`, `-` and `0` still Zoom the Source View alone.
