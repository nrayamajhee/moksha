#![feature(proc_macro_hygiene)]

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use cgmath::{
    perspective, Decomposed, Deg, InnerSpace, Matrix4, Quaternion, Rad, Rotation3, Vector3,
};
pub use genmesh::{
    generators::{Cube, IcoSphere, IndexedPolygon, SharedVertex},
    Triangulate, Vertex as GenVertex,
};
use maud::html;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{console, KeyboardEvent};

pub mod dom_factory;
use dom_factory::{add_event, body, document, request_animation_frame, window};

macro_rules! log {
    ( $( $t:tt )* ) => {
        use web_sys::console;
        use crate::dom_factory::document;
        use wasm_bindgen::JsValue;
		let document = document();
		let console_el = document.get_element_by_id("console");
		let msg: String = format!( $( $t )* ).into();
		match console_el {
			Some(el) => {
				if !el.class_list().contains("shown") {
					el.class_list().add_1("shown").unwrap();
				}
				console::log_1(&JsValue::from_str(&msg));
				let current = document.query_selector("#console p.current").unwrap();
				match current {
					Some(el) => {el.class_list().remove_1("current").unwrap();}
					_ => ()
				}
				el.clone().insert_adjacent_html("afterbegin", &format!("<p class='current'>{}</p>", msg)).unwrap();
			},
			None => {
				console::log_1(&JsValue::from("Couldn't find console element. Only displaying to the dev console!"));
				web_sys::console::log_1(&JsValue::from_str(&msg));
			}
		}
    }
}

pub mod renderer;
pub use renderer::{Viewport, Config, Renderer, ShaderType, Geometry, Material};

pub fn toggle_console(show: bool) {
    let console_el = document().get_element_by_id("console");
    match console_el {
        Some(el) => {
            if show {
                el.class_list().add_1("shown").unwrap();
            } else {
                el.class_list().remove_1("shown").unwrap();
            }
        }
        None => {
            log!("Couldn't find console element!");
        }
    }
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let dom = html! {
        canvas #gl-canvas {}
        section #console {button #close-console{"Close Console"}}
    };

    document().set_title("Webshell | Rayamajhee");
    body().set_inner_html(dom.into_string().as_str());

    let mut renderer = Renderer::new(Config {
        selector: "#gl-canvas",
        pixel_ratio: 1.,
    })
    .expect("Can't create a webgl renderer! Make sure your browser supports it!");

    let cube_geometry = Geometry::from_genmesh(&Cube::new());
    let sphere_geometry = Geometry::from_genmesh(&IcoSphere::subdivide(2));

    let mut colors = Vec::new();
    let face_colors = vec![
        [1.0, 1.0, 1.0, 1.0], // Front face: white
        [1.0, 0.0, 0.0, 1.0], // Back face: red
        [0.0, 1.0, 0.0, 1.0], // Top face: green
        [0.0, 0.0, 1.0, 1.0], // Bottom face: blue
        [1.0, 1.0, 0.0, 1.0], // Right face: yellow
        [1.0, 0.0, 1.0, 1.0], // Left face: purple
    ];
    for each in face_colors {
        for _ in 0..4 {
            for i in 0..4 {
                colors.push(each[i]);
            }
        }
    }
    let tex_coords = vec![
        // Front
        0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, // Back
        0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, // Top
        0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, // Bottom
        0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, // Right
        0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, // Left
        0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0,
    ];
    
    // let cube_material = Material::from_image_texture(renderer.get_context(), "/assets/img/box_tex.png", tex_coords)?;
    let cube_material = Material::color(renderer.get_context(), [1.0,1.0,0.0,1.0]);

    let mut r = 0.;
    let mut axis = Vector3::new(0.0, 1.0, 0.0);
    let mut transform: Decomposed<Vector3<f32>, Quaternion<f32>> = Decomposed {
        scale: 1.0,
        rot: Quaternion::from_axis_angle(axis, Rad::from(Deg(r))),
        disp: [-1.0, 0.0, 0.0].into(),
    };
    let model = Matrix4::from(transform);

    let proj = perspective(Rad::from(Deg(60.)), renderer.aspect_ratio(), 0.1, 100.);
    let view = Matrix4::look_at(
        [0.0, 3.0, 3.0].into(),
        [0.0, 0.0, 0.0].into(),
        Vector3::unit_y(),
    );
    let viewport = Viewport {
        proj,
        view,
    };
    renderer.bind_geometry(&cube_geometry)?;
    renderer.bind_material(&cube_material)?;
    renderer.update_transform(&model);
    renderer.update_viewport(&viewport);
    renderer.prepare_renderer();

    let a_rndr = Rc::new(RefCell::new(renderer));
    let b_rndr = a_rndr.clone();
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let mut renderer = a_rndr.borrow_mut();
        r = (r + 1.) % 360.;
        axis.x = axis.x + 1.;
        axis.y = axis.y + 2.;
        transform.rot = Quaternion::from_axis_angle(axis.normalize(), Rad::from(Deg(r)));
        let model = Matrix4::from(transform);
        // let buffers = a_buff.borrow();
        // let uniforms = a_unifs.borrow();
        // for (buffer, uniform) in buffers.iter().zip(uniforms.iter()) {
            // renderer
            //     .bind_buffers(buffer, uniform, "/assets/img/box_tex.png")
            //     .expect("Couldn't setup renderer");
            renderer
                .update_transform(&model);
            renderer.render().expect("Couldn't render!");
        // }
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));
    request_animation_frame(g.borrow().as_ref().unwrap());
    let window = window();
    add_event(&window, "resize", move |_| {
        let mut renderer = b_rndr.borrow_mut();
        renderer.resize();
    });
    add_event(
        &document().get_element_by_id("close-console").unwrap(),
        "click",
        move |_| {
            toggle_console(false);
        },
    );
    add_event(&window, "keydown", move |e| {
        if e.dyn_into::<KeyboardEvent>().unwrap().code() == "Backquote" {
            let console_el = document().get_element_by_id("console");
            match console_el {
                Some(el) => {
                    let shown = el.class_list().contains("shown");
                    toggle_console(!shown);
                }
                None => {
                    log!("Didn't find console element. Not adding event handlers!");
                }
            }
        }
    });
    Ok(())
}

    // let buffer = BufferObject {
    //     shader_type: ShaderType::Texture,
    //     vertices,
    //     indices,
    //     normals,
    //     color: Some([1.0, 0.0, 0.0, 1.0]),
    //     vertex_colors: Some(colors),
    //     tex_coords: Some(tex_coord),
    // };
    // let mut transform: Decomposed<Vector3<f32>, Quaternion<f32>> = Decomposed {
    //     scale: 1.0,
    //     rot: Quaternion::from_axis_angle(axis, Rad::from(Deg(r))),
    //     disp: [1.0, 0.0, 0.0].into(),
    // };
    // let model = Matrix4::from(transform);
    // let uniforms2 = UniformObject { model, view, proj };
    // let buffer2 = BufferObject {
    //     shader_type: ShaderType::Color,
    //     vertices,
    //     indices,
    //     normals,
    //     color: Some([1.0, 0.0, 0.0, 1.0]),
    //     vertex_colors: None,
    //     tex_coords: None,
    // };
    // let buffers = vec![buffer];
    // let uniforms = vec![uniforms];
    // // let buffers = vec![buffer, buffer2];
    // // let uniforms = vec![uniforms, uniforms2];

    // let buff = Rc::new(RefCell::new(buffers));
    // let unifs = Rc::new(RefCell::new(uniforms));
    // let a_buff = buff.clone();
    // let a_unifs = unifs.clone();
    // let b_unifs = unifs.clone();