#![feature(proc_macro_hygiene)]
#![doc(
    html_logo_url = "https://moksha.rayamajhee.com/assets/img/icon.png",
    html_favicon_url = "https://moksha.rayamajhee.com/assets/img/icon.png"
)]

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[macro_use]
extern crate serde_derive;

mod log_macro;
mod node_macro;

use std::cell::RefCell;
use std::rc::Rc;

/// Shorthand for Rc<RefCell\<T\>>.
pub type RcRcell<T> = Rc<RefCell<T>>;

/// Shorthand for Rc::new(RefCell::new(T)).
pub fn rc_rcell<T>(inner: T) -> RcRcell<T> {
    Rc::new(RefCell::new(inner))
}

pub mod controller;
pub mod dom_factory;
pub mod editor;
pub mod mesh;
pub mod renderer;
pub mod scene;

#[doc(inline)]
pub use crate::{
    controller::{MouseButton, ProjectionType, Viewport},
    editor::Editor,
    mesh::{Geometry, Material, Mesh, TextureType, Transform, Color},
    renderer::Renderer,
    scene::{Light, LightType, Node, ObjectInfo, Primitive, Scene, Storage},
};

mod start;
