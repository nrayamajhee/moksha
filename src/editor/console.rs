use maud::html;
use crate::dom_factory::{window, body, document, add_event};
use web_sys::KeyboardEvent;
use wasm_bindgen::JsCast;

pub fn setup() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    body().insert_adjacent_html("beforeend", markup().as_str()).expect("Couldn't insert console into the DOM!");
    add_events();
}
fn markup() -> String {
    let markup = html! {
        section #console {
            p {"Logs"}
            button#close-console {i.material-icons-round{"close"}}
        }
        button#open-console {i.material-icons-round{"assignment"} span.label {"Logs"} span.hint {"`"}}
    };
    markup.into_string()
}
fn add_events() {
    add_event(
        &document().get_element_by_id("close-console").unwrap(),
        "click",
        move |_| {
            toggle_console(false);
        },
    );

    add_event(
        &document().get_element_by_id("open-console").unwrap(),
        "click",
        move |_| {
            toggle_console(true);
        },
    );
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