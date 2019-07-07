#![feature(proc_macro_hygiene)]

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use cgmath::{
    perspective, Decomposed, Deg, InnerSpace, Matrix4, Quaternion, Rad, Rotation3, Vector3,
};
use genmesh::{
    generators::{Cone, Cube, Cylinder, IcoSphere, IndexedPolygon, SharedVertex, SphereUv, Torus},
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
        use crate::dom_factory::document;
        use web_sys::console;
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

pub mod controller;
pub mod mesh;
pub mod renderer;

use controller::Viewport;
use mesh::{Geometry, Material, Object, Storage, Mesh};
use renderer::{Renderer, ShaderType};

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

    let mut renderer = Renderer::new(renderer::Config {
        selector: "#gl-canvas",
        pixel_ratio: 1.,
    });

    let cube_geometry = Geometry::from_genmesh(&Cube::new());

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

    let cube_tex = Material::from_image_texture(
        renderer.get_context(),
        "/assets/img/box_tex.png",
        tex_coords,
    )?;


    let mesh_storage = Storage::new();
    let ms = Rc::new(RefCell::new(mesh_storage));

    let cube = Object::new(ms.clone(), cube_geometry.clone(), cube_tex);
    cube.set_position([-2.0, 0.0, 0.0]);
    
    let cube2 = Object::new(ms.clone(), cube_geometry.clone(), Material::vertex_colors(colors));
    cube2.set_position([2.0, 0.0, 0.0]);


    let sphere = Object::new(
        ms.clone(),
        Geometry::from_genmesh(&IcoSphere::subdivide(3)),
        Material::single_color([1.0, 0.0, 0.0, 1.0]),
    );
    sphere.set_position([0.0, 0.0, 5.0]);
    sphere.set_scale(2.0);

    let cone = Object::new(
        ms.clone(),
        Geometry::from_genmesh(&Cone::new(8)),
        Material::single_color([1.0, 1.0, 0.0, 1.0]),
    );
    cone.set_position([0.0, 0.0, -5.0]);

    let cylinder = Object::new(
        ms.clone(),
        Geometry::from_genmesh(&Cylinder::subdivide(8, 2)),
        Material::single_color([1.0, 0.0, 1.0, 1.0]),
    );
    cylinder.set_position([0.0, 0.0, -10.0]);

    let uv_sphere = Object::new(
        ms.clone(),
        Geometry::from_genmesh(&SphereUv::new(8, 16)),
        Material::single_color([0.0, 1.0, 1.0, 1.0]),
    );
    uv_sphere.set_position([5.0, 0.0, 0.0]);

    let torus = Object::new(
        ms.clone(),
        Geometry::from_genmesh(&Torus::new(2., 0.5, 8 , 8)),
        Material::single_color([0.0, 1.0, 1.0, 0.0]),
    );
    torus.set_position([-10.0, 0.0, 0.0]);


    let proj = perspective(Rad::from(Deg(60.)), renderer.aspect_ratio(), 0.1, 100.);
    let view = Matrix4::look_at(
        [10.0, 10.0, 12.0].into(),
        [0.0, 0.0, 0.0].into(),
        Vector3::unit_y(),
    );
    let viewport = Viewport { proj, view };
    {
        let ms = ms.clone();
        let storage = ms.borrow();
        renderer.setup_renderer(&storage);
        renderer.update_viewport(&viewport);
    }

    let window = window();
    let a_rndr = Rc::new(RefCell::new(renderer));
    let b_rndr = a_rndr.clone();
    let a_ms = ms.clone();
    let a_view = Rc::new(RefCell::new(viewport));
    let b_view = a_view.clone();
    let a_cube = Rc::new(RefCell::new(cube2));
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut r = 0.;
    let mut axis: Vector3<f32> = Vector3::unit_y();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let mut renderer = a_rndr.borrow_mut();
        r = (r + 1.) % 360.;
        axis.x += 1.;
        axis.y += 2.;
        let cube = a_cube.borrow_mut();
        cube.set_rotation(Quaternion::from_axis_angle(axis.normalize(), Rad::from(Deg(r))));
        renderer.render(&a_ms.borrow());
        renderer.update_viewport(&a_view.borrow());
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));
    request_animation_frame(g.borrow().as_ref().unwrap());
    add_event(&window, "resize", move |_| {
        let mut renderer = b_rndr.borrow_mut();
        let mut viewport = b_view.borrow_mut();
        renderer.resize(&mut viewport);
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
