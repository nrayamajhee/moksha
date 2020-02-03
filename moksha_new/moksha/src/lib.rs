mod log_macro;
pub mod mesh;
pub mod render;
pub mod scene;
pub mod storage;

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub type RcRcell<T> = Rc<RefCell<T>>;

pub fn rc_rcell<T>(inner: T) -> RcRcell<T> {
    Rc::new(RefCell::new(inner))
}

#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    use genmesh::generators::Cube;
    use mesh::{Mesh, Color, Geometry, Material};
    use scene::Scene;

    let scene = Scene::new();
    let cube = scene.mesh(
        "cube",
        Mesh {
            geometry: Geometry::from_genmesh(&Cube::new()),
            material: Material {
                color: Some(Color::white()),
            },
        },
    );
    log!("Storage" scene);
}
