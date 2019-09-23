mod scene;

pub mod primitives;

#[doc(inline)]
pub use primitives::{Gizmo, Primitive};
pub use scene::{Node, Scene, Storage};
