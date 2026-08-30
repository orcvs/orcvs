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
