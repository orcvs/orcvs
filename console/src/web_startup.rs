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
/// The `config.toml` text the page's `orcvs_config` element holds (ADR 0091),
/// or `None` when it holds none or its `data-media` query does not match.
///
pub(crate) fn page_settings() -> Option<String> {
    let window = web_sys::window()?;
    let element = window.document()?.get_element_by_id("orcvs_config")?;
    if let Some(media) = element.get_attribute("data-media") {
        let matches = window
            .match_media(&media)
            .ok()
            .flatten()
            .is_some_and(|list| list.matches());
        if !matches {
            return None;
        }
    }
    element.text_content()
}

///
/// This console's window, the page that frames it, and the console's own
/// origin, or `None` when no page frames it.
///
pub(crate) fn framing() -> Option<(web_sys::Window, web_sys::Window, String)> {
    let window = web_sys::window()?;
    let origin = window.location().origin().ok()?;
    let parent = window.parent().ok().flatten()?;
    if js_sys::Object::is(&parent, &window) {
        return None;
    }
    Some((window, parent, origin))
}

///
/// Listens for what the framing page posts this console, then posts the page
/// `{ type: "orcvs.ready" }`, before which the page sends nothing (ADR 0086,
/// ADR 0087). Only messages from the framing page's window and this
/// console's origin are read, and each repaints `ctx` so a frame answers it:
///
/// - `{ type: "orcvs.open", source }` keeps `source` in `inbox`, for a small
///   console. Only the latest Source opens, so a burst of messages holds one.
/// - `{ type: "orcvs.synth", available }` is the Built-in Synth's
///   announcement (`crate::built_in_synth`).
///
/// A console no page frames listens for nothing. The listener lasts as long
/// as the page.
///
pub(crate) fn listen_to_page(inbox: Option<crate::linked_source::Inbox>, ctx: egui::Context) {
    use wasm_bindgen::JsCast as _;
    use wasm_bindgen::closure::Closure;

    let Some((window, parent, origin)) = framing() else {
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
            if !from_parent || event.origin() != listened_origin {
                return;
            }
            if let Some(inbox) = &inbox
                && field(&data, "type").as_deref() == Some("orcvs.open")
                && let Some(source) = field(&data, "source")
            {
                *inbox.borrow_mut() = Some(source);
                ctx.request_repaint();
            } else if crate::built_in_synth::receive(&data) {
                ctx.request_repaint();
            }
        });
    if let Err(reason) =
        window.add_event_listener_with_callback("message", listener.as_ref().unchecked_ref())
    {
        log::warn!("the console cannot listen to its page: {reason:?}");
        return;
    }
    // The page holds the listener for its whole life.
    listener.forget();
    let ready = js_sys::Object::new();
    // Setting a field on a fresh plain object cannot fail.
    let _ = js_sys::Reflect::set(&ready, &"type".into(), &"orcvs.ready".into());
    if let Err(reason) = parent.post_message(&ready, &origin) {
        log::warn!("the console cannot say it is ready: {reason:?}");
    }
}
