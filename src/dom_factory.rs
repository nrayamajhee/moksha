use maud::{html, Markup};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    Document, Element, Event, EventTarget, HtmlCanvasElement, HtmlElement, Node, NodeList, Window,
};

pub fn window() -> Window {
    web_sys::window().expect("No global window found!")
}

pub fn document() -> Document {
    window().document().expect("Window has no document!")
}

pub fn body() -> HtmlElement {
    document().body().expect("Document has no body!")
}

use crate::rc_rcell;

pub fn loop_animation_frame<F>(closure: F)
where
    F: 'static + Fn(),
{
    let f = rc_rcell(None);
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        closure();
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));
    request_animation_frame(g.borrow().as_ref().unwrap());
}

pub fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

pub fn get_canvas(id: &str) -> HtmlCanvasElement {
    get_el(id)
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("Can't convert the dom element to HtmlCanvasElement!")
}

pub fn resize_canvas(canvas: &mut HtmlCanvasElement, pixel_ratio: f64) -> f32 {
    let window = window();
    let pixel_ratio = window.device_pixel_ratio() / pixel_ratio;
    let width: u32 = (window.inner_width().unwrap().as_f64().unwrap() * pixel_ratio) as u32;
    let height: u32 = (window.inner_height().unwrap().as_f64().unwrap() * pixel_ratio) as u32;
    canvas.set_width(width);
    canvas.set_height(height);
    width as f32 / height as f32
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

pub fn set_timeout<H>(callback: H, timeout: i32)
where
    H: 'static + Fn(),
{
    let cl = Closure::wrap(Box::new(callback) as Box<dyn Fn()>);
    window()
        .set_timeout_with_callback_and_timeout_and_arguments_0(cl.as_ref().unchecked_ref(), timeout)
        .unwrap();
    cl.forget();
}

pub fn get_el(id: &str) -> Element {
    document().get_element_by_id(id).unwrap()
}

pub fn get_html_el(id: &str) -> HtmlElement {
    document()
        .get_element_by_id(id)
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap()
}

pub fn query_html_el(selector: &str) -> HtmlElement {
    document()
        .query_selector(selector)
        .unwrap()
        .expect(&format!(
            "Can't find any element with query: `{}`",
            selector
        ))
        .dyn_into::<HtmlElement>()
        .expect("Can't cast the element as HtmlElement")
}

pub fn query_els(selector: &str) -> NodeList {
    document().query_selector_all(selector).unwrap()
}

pub fn icon_btn_w_id(id: &str, hint: &str, icon_name: &str, hotkey: &str) -> Markup {
    button(id, None, hint, icon_name, hotkey)
}

pub fn labelled_btn_w_id(
    id: &str,
    label: &str,
    hint: &str,
    icon_name: &str,
    hotkey: &str,
) -> Markup {
    button(id, Some(label), hint, icon_name, hotkey)
}

fn button(id: &str, label: Option<&str>, hint: &str, icon_name: &str, hotkey: &str) -> Markup {
    let class = match label {
        Some(_) => "labelled",
        None => "",
    };
    let label_span = html! {
        @if let Some(lbl) = label {
            span.label{(lbl)}
        }
    };
    html! {
        button class=(class) id=(id) aria-label=(hint) {i.material-icons-outlined{(icon_name)} (label_span) span.hint {(&format!("{} : {}",hint,hotkey))}}
    }
}

pub fn push_history(title: &str) -> Result<(), JsValue> {
    window().history().unwrap().push_state_with_url(
        &JsValue::from_str(title),
        "",
        Some(&format!("/{}", title)),
    )
}

pub fn replace_history(title: &str) -> Result<(), JsValue> {
    window().history().unwrap().replace_state_with_url(
        &JsValue::from_str(title),
        "",
        Some(&format!("/{}", title)),
    )
}

pub fn add_style(styles: &str) {
    let document = document();
    let style_el = document
        .create_element("style")
        .expect("Can't create style element");
    style_el.set_inner_html(styles);
    document
        .head()
        .expect("Can't get document's head")
        .dyn_into::<Node>()
        .unwrap()
        .append_child(&style_el)
        .expect("Can't attach style element into head");
}

pub fn get_target_innerh(e: &Event) -> String {
    e.target()
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap()
        .inner_html()
}

pub fn get_parent(node: Element, level: usize) -> Option<Element> {
    if level == 0 {
        return Some(node);
    }
    if let Some(parent) = node.parent_element() {
        get_parent(parent, level - 1)
    } else {
        None
    }
}

pub fn get_target_el(e: &Event) -> Element {
    e.target()
        .expect("No target element for the event")
        .dyn_into::<Element>()
        .unwrap()
}

pub fn get_target_parent_hel(e: &Event, level: usize) -> Element {
    get_parent(e.target().unwrap().dyn_into::<Element>().unwrap(), level)
        .expect("Could't get the element")
}

pub fn get_target_parent_el(e: &Event, level: usize) -> HtmlElement {
    get_parent(get_target_el(e), level)
        .expect("Could't get the element")
        .dyn_into::<HtmlElement>()
        .unwrap()
}
