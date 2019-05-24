//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]
use wasm_bindgen_test::*;

use webshell::Universe;
use webshell::Pattern;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_tick() {
    let mut input_universe = input_spaceship();
    let expected_universe = expected_spaceship();
    input_universe.tick();
    assert_eq!(&input_universe.get_cells(), &expected_universe.get_cells());
}

#[cfg(test)]
pub fn input_spaceship() -> Universe {
    let mut universe = Universe::new(6, 6);
    universe.insert(Pattern::GLIDER,1,1);
    universe
}

#[cfg(test)]
pub fn expected_spaceship() -> Universe {
    let mut universe = Universe::new(6, 6);
    universe.set_cells(&[(2, 1), (2, 3), (3, 2), (3, 3), (4, 2)]);
    universe
}
