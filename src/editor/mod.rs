pub mod console;

use crate::{
    dom_factory::{add_event, body, document, icon_btn_w_id, query_html_el, window},
    Viewport,
};
use maud::html;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, KeyboardEvent};

pub fn setup(view: Rc<RefCell<Viewport>>) {
    console_error_panic_hook::set_once();
    body()
        .insert_adjacent_html("beforeend", markup().as_str())
        .expect("Couldn't insert console into the DOM!");
    add_events(view);
}
fn markup() -> String {
    let markup = html! {
        section #toolbar {
            (icon_btn_w_id("toggle-perspective", "Switch Perspective", "crop_5_4", "P"))
            (icon_btn_w_id("zoom-in-out", "Zoom in/out view", "zoom_in", "Z"))
        }
    };
    markup.into_string()
}
fn handle_persp_toggle(a_view: Rc<RefCell<Viewport>>) {
    let icon = query_html_el("#toggle-perspective .material-icons-outlined");
    if icon.inner_html() == "panorama_horizontal" {
        icon.set_inner_html("crop_5_4");
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
    add_event(
        &document().get_element_by_id("zoom-in-out").unwrap(),
        "mousedown",
        move |_| {
            let mut view = a_view.borrow_mut();
            // view.enable_zoom();
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
