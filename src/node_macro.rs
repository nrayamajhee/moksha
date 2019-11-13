#[macro_export]
macro_rules! node {
    ($scene: expr, $mesh: expr, $($x:expr),*) => {
        {
            let node = $scene.from_mesh($mesh);
            use std::any::Any;
            use crate::{Mesh, ObjectInfo, renderer::{DrawMode, RenderFlags}};
            $(
                if let Some(name) = (&$x as &dyn Any).downcast_ref::<&str>() {
                    let mut info = node.info();
                    info.name = String::from(*name);
                    node.set_info(info);
                } else if let Some(name) = (&$x as &dyn Any).downcast_ref::<String>() {
                    let mut info = node.info();
                    info.name = String::from(name);
                    node.set_info(info);
                } else if let Some(info) = (&$x as &dyn Any).downcast_ref::<ObjectInfo>() {
                    node.set_info(info.to_owned());
                } else if let Some(mode) = (&$x as &dyn Any).downcast_ref::<DrawMode>() {
                    let mut info = node.info();
                    info.draw_mode = *mode;
                    node.set_info(info);
                } else if let Some(flags) = (&$x as &dyn Any).downcast_ref::<RenderFlags>() {
                    let mut info = node.info();
                    info.render_flags = *flags;
                    node.set_info(info);
                }
            )*
            node
        }
    }
}
