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
pub use game::{Game, DrawState, GameEvents, Controller};

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    let game = Game::new();
    game.attach_ui_elements();
    let g = Rc::new(RefCell::new(game));
    let controller = Controller::new(g);
    controller.attach_events();
    controller.render_loop();
}

