# 01 — Prototype the splash screen

Status: resolved
Blocked by: None — can start immediately

**What to build:** Throwaway prototypes of a splash screen showing the Orcvs logo, natively around
the real console and on the web page around the real WASM console, so Toby can choose by looking.

- [x] Native: three structurally different variants around the running console, chosen by an
      environment variable.
- [x] Web: three variants on the real `index.html`, switched by a URL parameter, in both
      appearances.
- [x] A candidate `logo-splash.png`.

## Answer

Native variant B, a splash window that becomes the console. Recorded in [`../spec.md`](../spec.md).

The variants were:

- A. In-window fade: the console window opens on the logo, which fades into the running Grid.
- B. Splash window: a borderless square with the logo, the name and the version, which grows into
  the console. **Chosen.**
- C. Welcome card: the Grid at once under a scrim, with the logo card held until the first input.

The prototype commits are on the `prototype/splash-screen` branch: `ORCVS_SPLASH=A|B|C|off` and
`ORCVS_SPLASH_MS` select and time the native variant; `?variant=A|B|C` and `?hold=0|1` drive the
web page. Captures are in the project files under `splash-screen/`.

The native prototype's B, trimmed to the decision (from the prototype):

```rust
// Opens borderless at SPLASH_WINDOW = [420.0, 460.0], centred, not resizable.
if leaving.is_none() && (opened.elapsed() >= hold || any_input) {
    ctx.send_viewport_cmd(ViewportCommand::Decorations(true));
    ctx.send_viewport_cmd(ViewportCommand::Resizable(true));
    ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(DEFAULT_VIEW_SIZE_MIN.into()));
    ctx.send_viewport_cmd(ViewportCommand::InnerSize(DEFAULT_VIEW_SIZE.into()));
    ctx.send_viewport_cmd(ViewportCommand::OuterPosition(centred_on_monitor));
}
// Then the logo fades over 350 ms, painted on a Foreground layer above the console's frame.
```
