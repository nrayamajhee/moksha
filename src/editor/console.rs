use crate::dom_factory::{add_event, body, document, icon_btn_w_id, labelled_btn_w_id, window};
use crate::log;
use maud::html;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;

pub fn setup(button: bool) {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    body()
        .insert_adjacent_html("beforeend", markup(button).as_str())
        .expect("Couldn't insert console into the DOM!");
    add_events(button);
}
fn markup(button: bool) -> String {
    let markup = html! {
        section #console {
            p {"Logs"}
            (icon_btn_w_id("close-console", "Close console", "close", "`"))
        }
        @if button {(labelled_btn_w_id("open-console", "Logs", "Open console", "assignment", "`"))}
    };
    markup.into_string()
}
fn add_events(button: bool) {
    add_event(
        &document().get_element_by_id("close-console").unwrap(),
        "click",
        move |_| {
            toggle_console(false);
        },
    );
    if button {
        add_event(
            &document().get_element_by_id("open-console").unwrap(),
            "click",
            move |_| {
                toggle_console(true);
            },
        );
    }
    add_event(&window(), "keydown", move |e| {
        let keycode = e.dyn_into::<KeyboardEvent>().unwrap().code();
        if keycode == "Backquote" {
            let console_el = document().get_element_by_id("console");
            match console_el {
                Some(el) => {
                    let shown = el.class_list().contains("shown");
                    toggle_console(!shown);
                }
                None => {
                    log!("Didn't find console element. Not adding event handlers!");
                }
            }
        }
    });
}
fn toggle_console(show: bool) {
    let console_el = document().get_element_by_id("console");
    match console_el {
        Some(el) => {
            if show {
                el.class_list().add_1("shown").unwrap();
            } else {
                el.class_list().remove_1("shown").unwrap();
            }
        }
        None => {
            log!("Couldn't find console element!");
        }
    }
}
