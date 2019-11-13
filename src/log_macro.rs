/// Logs to the developer console as well as editor's console panel
///
/// Please note that this macro doesn't require {:?} formatting.
/// Simply pass expressions separated by commas.

#[macro_export]
macro_rules! log {
    ($($x:expr),*) => {
        {
            let document = crate::dom_factory::document();
            let console_el = document.get_element_by_id("console");
            let mut msg = String::new();
            use std::any::Any;
            $(
                if let Some(s) = (&$x as &dyn Any).downcast_ref::<&str>() {
                    msg.push_str(&format!("{} ",s));
                } else {
                    if let Some(s) = (&$x as &dyn Any).downcast_ref::<String>() {
                        msg.push_str(&format!("{} ",s));
                    } else {
                        msg.push_str(&format!("{:?} ",$x));
                    }
                }
            )*
            match console_el {
                Some(_) => {
                    let log_el = document
                        .get_element_by_id("logs")
                        .unwrap();
                    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&msg));
                    log_el
                        .insert_adjacent_html(
                            "afterbegin",
                            &format!("<div><i class='material-icons-outlined'>info</i><pre>{}</pre></div>", msg),
                        )
                        .unwrap();
                },
                None => {
                    let msg = format!("dev console only: {:?}", msg);
                    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&msg));
                }
            }
        }
    };
}
