#![feature(proc_macro_hygiene)]

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

macro_rules! log {
    ( $( $t:tt )* ) => {
		let document = crate::dom_factory::document();
		let console_el = document.get_element_by_id("console");
		let msg: String = format!( $( $t )* ).into();
		match console_el {
			Some(_) => {
                let para_el = document.query_selector("#console p:first-of-type").unwrap().unwrap();
				web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&msg));
				para_el.insert_adjacent_html("afterend", &format!("<p><i class='material-icons-outlined'>info</i>{}</p>", msg)).unwrap();
			},
			None => {
                let msg = format!("dev console only: {:?}", msg);
				web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&msg));
			}
		}
    }
}

use genmesh::generators::{Cone, Cube, Cylinder, IcoSphere, Plane, SphereUv, Torus};
use maud::html;
use nalgebra::UnitQuaternion;
use std::cell::RefCell;
use std::f32::consts::PI;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlElement, KeyboardEvent, MouseEvent, Performance, WheelEvent};

pub mod dom_factory;
use dom_factory::{add_event, body, document, request_animation_frame, set_timeout, window};

pub mod controller;
pub mod editor;
pub mod mesh;
pub mod renderer;
pub mod scene;

use controller::{MouseButton, ProjectionConfig, ProjectionType, Viewport};
use editor::console;
use mesh::{Geometry, Material};
use renderer::{CursorType, DrawMode, Renderer};
use scene::{
    primitives::{create_transform_gizmo, ArrowType},
    Scene,
};

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console::setup();
    let dom = html! {
        canvas #gl-canvas oncontextmenu="return false;" {}
    };

    document().set_title("Webshell | Rayamajhee");
    body()
        .insert_adjacent_html("beforeend", dom.into_string().as_str())
        .expect("Couldn't insert markup into the DOM!");
    let window = window();

    let mut renderer = Renderer::new(renderer::Config {
        selector: "#gl-canvas",
        pixel_ratio: 1.0,
    });
    let viewport = Viewport::new(
        ProjectionConfig {
            fov: PI / 2.,
            near: 0.1,
            far: 100.,
        },
        renderer.aspect_ratio(),
        ProjectionType::Perspective,
    );

    let scene = Scene::new();

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

    let cube_tex =
        Material::from_image_texture(renderer.context(), "/assets/img/box_tex.png", tex_coords)?;

    // let cube = scene.object_from_mesh(cube_geometry.clone(), cube_tex);
    // let cube2 = scene.object_from_mesh(cube_geometry.clone(), Material::vertex_colors(colors));

    // cube.set_position([10.,0.,10.]);
    // cube2.set_position([-10.,0.,-10.]);
    // scene.add(&cube);
    // scene.add(&cube2);

    let grid = scene.object_from_mesh(
        Geometry::from_genmesh_no_normals(&Plane::subdivide(100, 100)),
        Material::single_color_no_shade(1.0, 1.0, 1.0, 1.0),
    );
    grid.scale(50.0);
    grid.set_rotation(UnitQuaternion::from_euler_angles(PI / 2., 0., 0.));
    scene.add_with_mode(&grid, DrawMode::Lines);

    let translation_gizmo = create_transform_gizmo(&scene, ArrowType::Cone);
    let scale_gizmo = create_transform_gizmo(&scene, ArrowType::Cube);
    let pan_gizmo = create_transform_gizmo(&scene, ArrowType::Sphere);
    scale_gizmo.set_position([8.0, 0.0, 0.0]);
    pan_gizmo.set_position([0.0, 0.0, 0.0]);
    //scene.add(&translation_gizmo);
    //scene.add(&scale_gizmo);
    scene.add(&pan_gizmo);

    let pan_gizmo = Rc::new(RefCell::new(pan_gizmo));

    let mut sun = scene.object_from_mesh(
        Geometry::from_genmesh(&IcoSphere::subdivide(1)),
        Material::single_color(1.0, 1.0, 0.0, 1.0),
    );

    let mut earth = scene.object_from_mesh(
        Geometry::from_genmesh(&IcoSphere::subdivide(1)),
        Material::single_color(0.0, 0.0, 1.0, 1.0),
    );

    let moon = scene.object_from_mesh(
        Geometry::from_genmesh(&IcoSphere::subdivide(1)),
        Material::single_color(1.0, 1.0, 1.0, 1.0),
    );

     moon.set_position([5.0, 0.0, 0.0]);
     earth.set_position([5.0, 0.0, 0.0]);
     moon.scale(0.5);
     earth.scale(0.5);
     sun.scale(2.0);

     let moon = Rc::new(RefCell::new(moon));
     earth.add(moon);
     let earth = Rc::new(RefCell::new(earth));
     sun.add(earth.clone());
     scene.add(&sun);

    renderer.setup_renderer(&scene);
    renderer.update_viewport(&viewport);

    let a_rndr = Rc::new(RefCell::new(renderer));
    let a_scene = Rc::new(RefCell::new(scene));
    let a_view = Rc::new(RefCell::new(viewport));
    // let a_cube = Rc::new(RefCell::new(cube));
    let a_sun = Rc::new(RefCell::new(sun));
    let a_earth = earth.clone();
    editor::setup(a_view.clone());

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    let p_g = pan_gizmo.clone();

    let b_rndr = a_rndr.clone();
    let b_view = a_view.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let mut renderer = b_rndr.borrow_mut();
         let sun = a_sun.borrow();
         let earth = a_earth.borrow();
        // let cube = a_cube.borrow_mut();
        // cube.rotate_by(UnitQuaternion::from_euler_angles(0.01, 0.02, 0.));
        earth.rotate_by(UnitQuaternion::from_euler_angles(0.,0.02, 0.));
        sun.rotate_by(UnitQuaternion::from_euler_angles(0.,0.01, 0.));
        renderer.render(&a_scene.borrow());
        renderer.update_viewport(&b_view.borrow());
        let pan_gizmo = p_g.borrow_mut();
        let transform = &b_view.borrow().get_transform();
        let p = transform.rotation.transform_vector(&transform.translation.vector);
        //let p = -transform.translation.vector;
        pan_gizmo.set_position([p.x,p.y,p.z]);
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    let b_rndr = a_rndr.clone();
    let b_view = a_view.clone();
    add_event(&window, "resize", move |_| {
        let mut renderer = b_rndr.borrow_mut();
        let mut viewport = b_view.borrow_mut();
        renderer.resize(&mut viewport);
        viewport.resize(renderer.aspect_ratio());
    });

    let b_view = a_view.clone();
    let b_rndr = a_rndr.clone();
    let perf = window.performance().unwrap();
    add_event(&b_rndr.borrow().canvas(), "mousemove", move |e| {
        let mut view = b_view.borrow_mut();
        let me = e.dyn_into::<MouseEvent>().unwrap();
        let dt = perf.now();
        view.update_rot(me.movement_x(), me.movement_y(), dt as f32);
        view.update_zoom(me.movement_y() as f32);
    });

    if let Some(button) = a_view.borrow().button() {
        let renderer = a_rndr.borrow();
        let canvas = renderer.canvas();
        let b_view = a_view.clone();
        let b_rndr = a_rndr.clone();
        add_event(canvas, "mousedown", move |e| {
            let mut view = b_view.borrow_mut();
            let renderer = b_rndr.borrow_mut();
            let me = e.dyn_into::<MouseEvent>().unwrap();
            if me.button() == button as i16 {
                renderer.change_cursor(CursorType::Grab);
                view.enable_rotation();
            }
        });
        let b_view = a_view.clone();
        let b_rndr = a_rndr.clone();
        add_event(&window, "mouseup", move |e| {
            let mut view = b_view.borrow_mut();
            let renderer = b_rndr.borrow_mut();
            let me = e.dyn_into::<MouseEvent>().unwrap();
            if me.button() == button as i16 {
                renderer.change_cursor(CursorType::Pointer);
                view.disable_rotation();
                view.disable_zoom()
            }
        });
    }

    let b_view = a_view.clone();
    let b_rndr = a_rndr.clone();
    add_event(&b_rndr.borrow().canvas(), "wheel", move |e| {
        let mut view = b_view.borrow_mut();
        let we = e.dyn_into::<WheelEvent>().unwrap();
        let dy = we.delta_y() as f32;
        view.enable_zoom();
        view.update_zoom(dy);
        view.disable_zoom();
    });

    let b_view = a_view.clone();
    let b_rndr = a_rndr.clone();
    add_event(&window, "keydown", move |e| {
        let keycode = e.dyn_into::<KeyboardEvent>().unwrap().code();
        if keycode == "KeyR" {
            let mut view = b_view.borrow_mut();
            view.reset();
        }
    });
    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}
