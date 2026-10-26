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
/// Keeps the Source File text of the latest `{ type: "orcvs.open", source }`
/// message the embedding page posts in `inbox`, and repaints `ctx` so a frame
/// opens it (ADR 0086). A message from another window or origin, of another
/// type, or whose `source` is not a string is ignored. Once listening, posts
/// `{ type: "orcvs.ready" }` to the embedding page, which sends nothing
/// before it. The listener lasts as long as the page.
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
    let Ok(Some(parent)) = window.parent() else {
        return;
    };
    let field = |data: &wasm_bindgen::JsValue, name: &str| {
        js_sys::Reflect::get(data, &name.into())
            .ok()
            .and_then(|value| value.as_string())
    };
    let listened_parent = parent.clone();
    let listened_origin = origin.clone();
    let listener =
        Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |event: web_sys::MessageEvent| {
            let data = event.data();
            let from_parent = event
                .source()
                .is_some_and(|sender| js_sys::Object::is(&sender, &listened_parent));
            if !from_parent
                || event.origin() != listened_origin
                || field(&data, "type").as_deref() != Some("orcvs.open")
            {
                return;
            }
            // Only the latest Source opens, so a burst of messages holds one.
            if let Some(source) = field(&data, "source") {
                *inbox.borrow_mut() = Some(source);
                ctx.request_repaint();
            }
        });
    if let Err(reason) =
        window.add_event_listener_with_callback("message", listener.as_ref().unchecked_ref())
    {
        log::warn!("the small console cannot listen for Sources: {reason:?}");
        return;
    }
    let ready = js_sys::Object::new();
    // Setting a field on a fresh plain object cannot fail.
    let _ = js_sys::Reflect::set(&ready, &"type".into(), &"orcvs.ready".into());
    if let Err(reason) = parent.post_message(&ready, &origin) {
        log::warn!("the small console cannot say it is ready: {reason:?}");
    }
    // The page holds the listener for its whole life.
    listener.forget();
}
