pub mod console;

use maud::html;
use crate::{dom_factory::{icon_btn_w_id, window, body, document, add_event, query_html_el}, Viewport};
use web_sys::{KeyboardEvent, HtmlElement};
use wasm_bindgen::JsCast;
use std::cell::RefCell;
use std::rc::Rc;

pub fn setup(view: Rc<RefCell<Viewport>>) {
    console_error_panic_hook::set_once();
    body().insert_adjacent_html("beforeend", markup().as_str()).expect("Couldn't insert console into the DOM!");
    add_events(view);
}
fn markup() -> String {
    let markup = html! {
        section #toolbar {
            (icon_btn_w_id("toggle-perspective", "Switch Perspective", "crop_16_9", "P"))
            // button#toggle-perspective {i.material-icons-round{"crop_16_9"} span.hint {"Switch Perspective: P"}}
        }
    };
    markup.into_string()
}
fn handle_persp_toggle(a_view: Rc<RefCell<Viewport>>) {
    let icon = query_html_el("#toggle-perspective .material-icons-round");
    if icon.inner_html() == "panorama_horizontal" {
        icon.set_inner_html("crop_16_9");
    } else {
        icon.set_inner_html("panorama_horizontal");
    }
    let mut view = a_view.borrow_mut();
    view.switch_projection();
}
fn add_events(view: Rc<RefCell<Viewport>>) {
    let a_view = view.clone();
    add_event(
        &document().get_element_by_id("toggle-perspective").unwrap(),
        "click",
        move |_| {
            handle_persp_toggle(a_view.clone());
        },
    );
    let a_view = view.clone();
    add_event(&window(), "keydown", move |e| {
        let keycode = e.dyn_into::<KeyboardEvent>().unwrap().code();
        if keycode == "KeyP" {
            handle_persp_toggle(a_view.clone());
        }
    });
}