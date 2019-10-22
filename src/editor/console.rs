use crate::dom_factory::{
    add_event, body, document, icon_btn_w_id, labelled_btn_w_id, push_history, replace_history,
    window,
};

use crate::log;
use maud::html;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;

#[derive(Debug, Clone, Copy)]
pub struct ConsoleConfig {
    pub ui_button: bool,
    pub change_history: bool,
}

pub fn console_setup(config: ConsoleConfig) {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    body()
        .insert_adjacent_html("beforeend", markup(config.ui_button).as_str())
        .expect("Couldn't insert console into the DOM!");
    add_events(config);
}
fn markup(button: bool) -> String {
    let markup = html! {
        section #console {
            div.header {
                p {"Logs"}
                (icon_btn_w_id("close-console", "Close console", "close", "`"))
            }
            section #logs {
            }
        }        @if button {(labelled_btn_w_id("open-console", "Logs", "Open console", "assignment", "`"))}
    };
    markup.into_string()
}
fn add_events(config: ConsoleConfig) {
    add_event(
        &document().get_element_by_id("close-console").unwrap(),
        "click",
        move |_| {
            toggle_console(false, config.change_history);
        },
    );
    if config.ui_button {
        add_event(
            &document().get_element_by_id("open-console").unwrap(),
            "click",
            move |_| {
                toggle_console(true, config.change_history);
            },
        );
    }
    let window = window();
    add_event(&window, "keydown", move |e| {
        let keycode = e.dyn_into::<KeyboardEvent>().unwrap().code();
        if keycode == "Backquote" {
            let console_el = document().get_element_by_id("console");
            match console_el {
                Some(el) => {
                    let shown = el.class_list().contains("shown");
                    toggle_console(!shown, config.change_history);
                }
                None => {
                    log!("Didn't find console element. Not adding event handlers!");
                }
            }
        }
    });
    if config.change_history {
        let storage = window.session_storage().unwrap().unwrap();
        if let Some(redirect) = storage.get_item("redirect").unwrap() {
            if redirect.as_str() == "/console" {
                toggle_console(true, true);
                storage.remove_item("redirect").unwrap();
            }
        }
        add_event(&window, "popstate", |_| {
            if document().location().unwrap().pathname().unwrap().as_str() == "/console" {
                toggle_console(true, true);
            } else {
                toggle_console(false, true);
            }
        });
    }
}
pub fn toggle_console(show: bool, history: bool) {
    let console_el = document().get_element_by_id("console");
    match console_el {
        Some(el) => {
            if show {
                if history {
                    push_history("console").unwrap_or_else(|err| {
                        log!("Couldn't modify history: ", err);
                    });
                }
                el.class_list().add_1("shown").unwrap();
            } else {
                if history {
                    replace_history("").unwrap_or_else(|err| {
                        log!("Couldn't modify history: ", err);
                    });
                }
                el.class_list().remove_1("shown").unwrap();
            }
        }
        None => {
            log!("Couldn't find console element!");
        }
    }
}
