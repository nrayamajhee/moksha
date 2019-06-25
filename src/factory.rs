use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Event, EventTarget, HtmlElement, Window};

pub fn window() -> Window {
    web_sys::window().expect("No global window found!")
}

pub fn document() -> Document {
    window().document().expect("Window has no document!")
}

pub fn body() -> HtmlElement {
    document().body().expect("Document has no body!")
}

pub fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

pub fn add_event<H>(target: &EventTarget, event_type: &str, event_listener: H)
where
    H: 'static + FnMut(Event),
{
    let cl = Closure::wrap(Box::new(event_listener) as Box<dyn FnMut(_)>);
    target
        .add_event_listener_with_callback(event_type, cl.as_ref().unchecked_ref())
        .unwrap();
    cl.forget();
}
