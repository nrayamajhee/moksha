#[macro_export]
macro_rules! log {
    ($($x:expr) *) => {
        {
            let mut msg = String::new();
            use std::any::Any;
            $(
                if let Some(s) = (&$x as &dyn Any).downcast_ref::<&str>() {
                    msg.push_str(&format!("{} ",s));
                } else if let Some(s) = (&$x as &dyn Any).downcast_ref::<String>() {
                    msg.push_str(&format!("{} ",s));
                } else {
                    msg.push_str(&format!("{:?} ",$x));
                }
            )*
            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&msg));
        }
    };
}
