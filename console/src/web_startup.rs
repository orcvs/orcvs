use web_sys::{Element, HtmlCanvasElement};

pub const MISSING_CANVAS_MESSAGE: &str =
    "<p> The app could not find its drawing canvas. See the developer console for details. </p>";

pub fn canvas_or_report(
    canvas: Option<HtmlCanvasElement>,
    loading_text: Option<Element>,
) -> Option<HtmlCanvasElement> {
    if canvas.is_none()
        && let Some(loading_text) = loading_text
    {
        loading_text.set_inner_html(MISSING_CANVAS_MESSAGE);
    }
    canvas
}

///
/// Puts the Source File text each `{ type: "orcvs.open", source }` message
/// posts in `inbox`, and repaints `ctx` so a frame opens it (ADR 0086). A
/// message from another origin, of another type, or whose `source` is not a
/// string is ignored. The listener lasts as long as the page.
///
pub(crate) fn listen_for_sources(inbox: crate::linked_source::Inbox, ctx: egui::Context) {
    use wasm_bindgen::JsCast as _;
    use wasm_bindgen::closure::Closure;

    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(origin) = window.location().origin() else {
        return;
    };
    let field = |data: &wasm_bindgen::JsValue, name: &str| {
        js_sys::Reflect::get(data, &name.into())
            .ok()
            .and_then(|value| value.as_string())
    };
    let listener =
        Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |event: web_sys::MessageEvent| {
            let data = event.data();
            if event.origin() != origin || field(&data, "type").as_deref() != Some("orcvs.open") {
                return;
            }
            if let Some(source) = field(&data, "source") {
                inbox.borrow_mut().push(source);
                ctx.request_repaint();
            }
        });
    if let Err(reason) =
        window.add_event_listener_with_callback("message", listener.as_ref().unchecked_ref())
    {
        log::warn!("the small console cannot listen for Sources: {reason:?}");
    }
    // The page holds the listener for its whole life.
    listener.forget();
}
