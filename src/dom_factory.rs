use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{
    Document, Event, HtmlBodyElement, HtmlElement, HtmlHeadElement, HtmlStyleElement, MouseEvent,
    HtmlInputElement, Node
};

pub fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

pub fn document() -> Document {
    window().document().expect("`window` has no document")
}

pub fn head() -> HtmlHeadElement {
    document().head().expect("`document` has no head")
}

pub fn body() -> HtmlBodyElement {
    document()
        .body()
        .expect("`document` has no body")
        .dyn_into::<HtmlBodyElement>()
        .expect("Can't cast html element to body element")
}

pub fn min_max_input(input_type: &str, min: i32, max: i32, val: i32, delta: Option<u32>) -> HtmlElement {
    let input = document()
        .create_element("input")
        .expect("Can't create a input element.");
    input.set_attribute("type", input_type).unwrap();
    if let Some(delta) = delta {
        input.set_attribute("step", delta.to_string().as_str()).unwrap();
    }
    let i = input.dyn_into::<HtmlInputElement>().unwrap();
    i.set_min(min.to_string().as_str());
    i.set_max(max.to_string().as_str());
    i.set_value(val.to_string().as_str());
    i
        .dyn_into::<HtmlElement>()
        .expect("Can't cast a element as HtmlElement.")
}

pub fn button(text: &str, icon_type: &str) -> HtmlElement {
    let btn = document()
        .create_element("button")
        .expect("Can't create a button element.")
        .dyn_into::<HtmlElement>()
        .expect("Can't cast a element as HtmlElement.");
    if icon_type == "material_icon" {
        btn.set_inner_html(format!("<i class='material-icons'>{}</i>", text).as_str());
    } else {
        btn.set_inner_html(text);
    }
    btn
}

pub fn style_sheet(text: &str) -> HtmlStyleElement {
    let style_sheet = document()
        .create_element("style")
        .expect("Can't create a style element.")
        .dyn_into::<HtmlStyleElement>()
        .expect("Can't cast a element as HtmlStyleElement.");
    style_sheet.set_type("text/css");
    style_sheet.set_inner_html(text);
    style_sheet
}

pub fn add_style(text: &str) {
    let css = style_sheet(text);
    head().append_child(&css.dyn_into::<Node>().expect("Can't cast the stylesheet as node!"))
        .expect("Can't add the css to `head`");
}

pub fn request_animation_frame(f: &Closure<FnMut()>) -> i32 {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("Should register `requestAnimationFrame`")
}

pub fn cancel_animation_frame(i: i32) {
    window()
        .cancel_animation_frame(i)
        .expect(format!("Can't cancel animation frame with id: {}", i).as_str());
}

pub fn add_event<H>(el: &HtmlElement, event_type: &str, event_listener: H)
where
    H: 'static + FnMut(Event),
{
    let cl = Closure::wrap(Box::new(event_listener) as Box<dyn FnMut(_)>);
    el.add_event_listener_with_callback(event_type, cl.as_ref().unchecked_ref())
        .unwrap();
    cl.forget();
}

pub fn set_timeout<F>(callback: F, time: u32) -> i32 where F: 'static + FnMut()  {
    let cl = Closure::wrap(Box::new(callback) as Box<dyn FnMut()>);
    let id = window().set_timeout_with_callback_and_timeout_and_arguments_0(cl.as_ref().unchecked_ref(), time as i32).unwrap();
    cl.forget();
    id
}
