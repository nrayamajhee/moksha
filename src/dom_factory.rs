use crate::{rc_rcell, editor::fps};
use maud::{html, Markup};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{
    Document, Element, Event, EventTarget, FileList, FileReader, HtmlCanvasElement, HtmlCollection,
    HtmlElement, HtmlInputElement, Node, NodeList, ProgressEvent, Window,
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

pub fn loop_animation_frame<F>(closure: F, fps: Option<f64>)
where
    F: 'static + Fn(),
{
    let f = rc_rcell(None);
    let g = f.clone();
    let fps_viewer = fps::setup(fps);
    let fps_v = fps_viewer.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let then = now();
        closure();
        if let Some(fps) = fps {
            let delay = (1000. / fps - (now() - then)) as i32;
            let h = f.clone();
            set_timeout(move || {
                fps::log(1000. / (now() - then));
                request_animation_frame(h.borrow().as_ref().unwrap());
            }, delay);
        } else {
            fps::log(1000. / (now() - then));
            request_animation_frame(f.borrow().as_ref().unwrap());
        }
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

pub fn resize_canvas(canvas: &HtmlCanvasElement, pixel_ratio: f64) -> f64 {
    let window = window();
    let pixel_ratio = window.device_pixel_ratio() / pixel_ratio;
    let width = window.inner_width().unwrap().as_f64().unwrap() * pixel_ratio;
    let height = window.inner_height().unwrap().as_f64().unwrap() * pixel_ratio;
    canvas.set_width(width as u32);
    canvas.set_height(height as u32);
    width / height
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

pub fn now() -> f64 {
    window()
        .performance()
        .expect("Performance should be available")
        .now()
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
    document()
        .get_element_by_id(id)
        .unwrap_or_else(|| panic!("Cant find element with id {}", id))
}

pub fn get_el_by_class(class: &str) -> Element {
    document()
        .get_elements_by_class_name(class)
        .item(0)
        .unwrap_or_else(|| panic!("Cant find element with class {}", class))
}

pub fn get_els_by_class(class: &str) -> HtmlCollection {
    document().get_elements_by_class_name(class)
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
        .unwrap_or_else(|_| panic!("Can't find any element with query: `{}`", selector))
        .unwrap()
        .dyn_into::<HtmlElement>()
        .expect("Can't cast the element as HtmlElement")
}

pub fn query_els(selector: &str) -> NodeList {
    document()
        .query_selector_all(selector)
        .unwrap_or_else(|_| panic!("No element matches selector: {}", selector))
}

pub fn query_el(selector: &str) -> Element {
    document()
        .query_selector(selector)
        .unwrap()
        .unwrap_or_else(|| panic!("No element matches selector: {}", selector))
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

pub fn create_el(name: &str) -> Element {
    document()
        .create_element(name)
        .unwrap_or_else(|_| panic!("Can't create element with name {}", name))
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

pub fn el_innerh(e: Element) -> String {
    e.dyn_into::<HtmlElement>().unwrap().inner_html()
}

pub fn get_parent(node: &Element, level: usize) -> Option<Element> {
    if level == 0 {
        return Some(node.clone());
    }
    if let Some(parent) = node.parent_element() {
        get_parent(&parent, level - 1)
    } else {
        None
    }
}

pub fn get_target(e: &Event) -> EventTarget {
    e.target().expect("No target element for the event!")
}

pub fn get_target_innerh(e: &Event) -> String {
    get_target(&e)
        .dyn_into::<HtmlElement>()
        .unwrap()
        .inner_html()
}

pub fn get_target_el(e: &Event) -> Element {
    get_target(&e)
        .dyn_into::<Element>()
        .expect("Can't cast as Element!")
}

pub fn get_target_files(e: &Event) -> FileList {
    get_target(&e)
        .dyn_into::<HtmlInputElement>()
        .expect("Can't cast as HtmlInputElement! This might not be an input element.")
        .files()
        .expect("No files in the input element")
}

pub fn get_target_file_result(e: &Event) -> String {
    get_target(&e)
        .dyn_into::<FileReader>()
        .expect("Can't cast as File Reader!")
        .result()
        .expect("File reader has no result content!")
        .as_string()
        .expect("Can't parse reader result as string!")
}

pub fn get_progress(e: Event) -> ProgressEvent {
    e.dyn_into::<ProgressEvent>()
        .expect("Can't cast event as ProgrssEvent")
}

pub fn get_target_parent_el(e: &Event, level: usize) -> Element {
    get_parent(&get_target_el(&e), level).expect("Could't get the element!")
}

pub fn get_target_parent_html_el(e: &Event, level: usize) -> HtmlElement {
    get_parent(&get_target_el(&e), level)
        .expect("Could't get the element")
        .dyn_into::<HtmlElement>()
        .unwrap()
}

pub fn create_el_w_class_n_inner(tag_name: &str, class: &str, inner_html: &str) -> Element {
    let el = document()
        .create_element(tag_name)
        .unwrap_or_else(|_| panic!("Can't create element with tage name: {}", tag_name));
    for each in class.split(' ') {
        el.class_list().add_1(each).expect("Can't add class name");
    }
    el.set_inner_html(inner_html);
    el
}

pub fn add_class(el: &Element, class_name: &str) {
    el.class_list()
        .add_1(class_name)
        .unwrap_or_else(|_| panic!("Can't add class name: {} to element: {:?}", class_name, el));
}
pub fn remove_class(el: &Element, class_name: &str) {
    el.class_list()
        .remove_1(class_name)
        .unwrap_or_else(|_| panic!("Can't add class name: {} to element: {:?}", class_name, el));
}
pub fn insert_el(parent: &Element, child: &Element) {
    insert_el_at(parent, child, "beforeend");
}
pub fn insert_el_at(parent: &Element, child: &Element, option: &str) {
    parent
        .insert_adjacent_element(option, &child)
        .expect("Couldn't insert adjacent element before end!");
}
