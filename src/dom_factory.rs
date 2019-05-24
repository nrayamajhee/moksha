use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{
        Document, Event, HtmlBodyElement, Element, HtmlElement, HtmlHeadElement, HtmlStyleElement, Node,
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

pub fn get_el_id(id: &str) -> HtmlElement {
        document()
                .get_element_by_id(id)
                .expect(format!("Can't find element with id: {}", id).as_str())
                .dyn_into::<HtmlElement>()
                .unwrap()
}

pub fn query(query: &str) -> HtmlElement {
        document()
                .query_selector(query)
                .unwrap()
                .expect(format!("Can't find element with query: {}", query).as_str())
                .dyn_into::<HtmlElement>()
                .unwrap()
}

pub fn get_css_var(var: &str) -> String {
        window().get_computed_style(
                &document().document_element().unwrap()
        ).expect("Can't read style of the document").unwrap()
        .get_property_value(var).unwrap()
}

pub fn add_style(text: &str) {
        let css = style_sheet(text);
        head().append_child(
                &css.dyn_into::<Node>()
                        .expect("Can't cast the stylesheet as node!"),
        )
        .expect("Can't add the css to `head`");
}

pub fn request_animation_frame(f: &Closure<FnMut()>) -> i32 {
        window().request_animation_frame(f.as_ref().unchecked_ref())
                .expect("Should register `requestAnimationFrame`")
}

pub fn cancel_animation_frame(i: i32) {
        window().cancel_animation_frame(i).expect(format!(
                "Can't cancel animation frame with id: {}",
                i
        )
        .as_str());
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

pub fn set_timeout<F>(callback: F, time: u32) -> i32
where
        F: 'static + FnMut(),
{
        let cl = Closure::wrap(Box::new(callback) as Box<dyn FnMut()>);
        let id = window()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                        cl.as_ref().unchecked_ref(),
                        time as i32,
                )
                .unwrap();
        cl.forget();
        id
}
