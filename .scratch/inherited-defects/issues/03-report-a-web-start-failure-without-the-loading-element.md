# 03 — Report a web start failure without the loading element

**What to fix:** In the WASM entry point, the check of `start_result` sits inside
`if let Some(loading_text) = loading_text`. If the page has no `#loading_text` element, the shell
drops a failed start without a log, a panic, or a message.

**Status:** ready-for-agent
**Implementation:** complete

- [x] A failed `WebRunner::start` is always reported, with or without the loading element.
- [x] The loading element still shows the crash message when the element is present.
- [x] The loading element is still removed on a successful start.

## Comments

`shell/src/main.rs:90-104`. The block reads:

```rust
if let Some(loading_text) = loading_text {
    match start_result {
        Ok(_) => loading_text.remove(),
        Err(e) => { /* set message, then panic */ }
    }
}
```

The element controls whether the result is examined at all. That is the wrong order. The result is
always worth reporting. The element only decides where to show it.

Today the element exists, so the defect is latent. It becomes live if `index.html` renames the
element or drops it, or if a future change removes the element earlier in start-up. The user then
gets a blank canvas, and the browser console holds nothing.

Invert the two. Match on `start_result` first. Log the error and panic in the `Err` arm. Update the
element inside each arm, only when it is present.

`eframe::WebLogger` is already installed at line 63, so `log::error!` reaches the browser console.
