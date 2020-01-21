#[macro_export]
macro_rules! node_from_obj_wired {
    ($scene: expr, $dir: expr, $file: expr) => {{
        $scene.object_from_obj(
            $dir,
            include_str!(concat!($dir, "/", $file, ".obj")),
            Some(include_str!(concat!($dir, "/", $file, ".mtl"))),
            None,
            true,
        )
    }};
}
#[macro_export]
macro_rules! node_from_obj {
    ($scene: expr, $dir: expr, $file: expr) => {{
        $scene.object_from_obj(
            $dir,
            include_str!(concat!($dir, "/", $file, ".obj")),
            Some(include_str!(concat!($dir, "/", $file, ".mtl"))),
            None,
            false,
        )
    }};
}
#[macro_export]
macro_rules! object {
    ($scene: expr, $mesh: expr, $($x:expr),*) => {
        {
            let mut setup_unique_vertices = false;
            $(
                if let Some(mode) = (&$x as &dyn Any).downcast_ref::<DrawMode>() {
                    if *mode == DrawMode::Arrays {
                        setup_unique_vertices = true;
                    }
                }
            )*
            let object = $scene.from_mesh($mesh, setup_unique_vertices);
            use std::any::Any;
            use crate::{ ObjectInfo, renderer::{DrawMode, RenderFlags}};
            $(
                if let Some(name) = (&$x as &dyn Any).downcast_ref::<&str>() {
                    let mut info = object.info();
                    info.name = String::from(*name);
                    object.set_info(info);
                } else if let Some(name) = (&$x as &dyn Any).downcast_ref::<String>() {
                    let mut info = object.info();
                    info.name = String::from(name);
                    object.set_info(info);
                } else if let Some(info) = (&$x as &dyn Any).downcast_ref::<ObjectInfo>() {
                    object.set_info(info.to_owned());
                } else if let Some(mode) = (&$x as &dyn Any).downcast_ref::<DrawMode>() {
                    let mut info = object.info();
                    info.draw_mode = *mode;
                    object.set_info(info);
                } else if let Some(flags) = (&$x as &dyn Any).downcast_ref::<RenderFlags>() {
                    let mut info = object.info();
                    info.render_flags = *flags;
                    object.set_info(info);
                }
            )*
            object
        }
    }
}
