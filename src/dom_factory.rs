use wasm_bindgen::{prelude::*, JsCast, JsValue};
use web_sys::{Document, EventTarget, HtmlElement};

pub fn button(text: &str, icon_type: &str) -> HtmlElement {
    let btn = document()
        .create_element("button")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();
    if icon_type == "material_icon" {
        btn.set_inner_html(format!("<i class='material-icons'>{}</i>", text).as_str());
    } else {
        btn.set_inner_html(text);
    }
    btn
}

pub fn document() -> Document {
    window().document().expect("`window` has no document")
}

pub fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

pub fn request_animation_frame(f: &Closure<FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame`");
}

pub fn add_events(el: &HtmlElement, event_type: &str, event_listener: Box<FnMut()>) {
    let cl = Closure::wrap(event_listener as Box<dyn FnMut()>);
    el.add_event_listener_with_callback("click", cl.as_ref().unchecked_ref())
        .unwrap();
    cl.forget();
}
