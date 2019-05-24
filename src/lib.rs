#![feature(proc_macro_hygiene)]

#[allow(unused_macros)]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

mod universe;
pub use universe::{Universe, Pattern};

mod game;
pub mod dom_factory;
pub use game::{Game, DrawState, Controller};

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    let game = Game::new();
    let controller = Controller::new(Rc::new(RefCell::new(game)));
    controller.attach_events();
    controller.render_loop();
}

