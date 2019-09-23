#![feature(proc_macro_hygiene)]
#![doc(html_logo_url = "https://moksha.rayamajhee.com/assets/img/icon.png", html_favicon_url = "https://moksha.rayamajhee.com/assets/img/icon.png")]

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[macro_use]
mod log_macros;

use std::rc::Rc;
use std::cell::RefCell;
pub type RcRcell<T> = Rc<RefCell<T>>;

pub fn rc_rcell<T>(inner: T) -> RcRcell<T> {
   Rc::new(RefCell::new(inner))
}

use genmesh::generators::{Cone, Cube, Cylinder, IcoSphere, Plane, SphereUv, Torus};
use maud::html;
use nalgebra::UnitQuaternion;
use std::f32::consts::PI;
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

#[doc(inline)]
pub use crate::{
    scene::{Node, Scene, Storage, Primitive},
    renderer::Renderer,
    mesh::{Geometry, Material, Mesh, Transform},
    editor::Editor,
    controller::{Viewport,ProjectionType, MouseButton},
};

use controller::{ProjectionConfig};
use renderer::{CursorType, DrawMode};
use scene::{
    primitives::{create_transform_gizmo, ArrowType},
};
use editor::console_setup;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_setup(true);
    let dom = html! {
        canvas #gl-canvas oncontextmenu="return false;" {}
    };
    document().set_title("Editor | Moksha");
    body()
        .insert_adjacent_html("beforeend", dom.into_string().as_str())
        .expect("Couldn't insert markup into the DOM!");
    let window = window();

    let renderer = Renderer::new(renderer::Config {
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

    let a_rndr = rc_rcell(renderer);
    let scene = Scene::new(a_rndr.clone());

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
    Material::from_image_texture(a_rndr.borrow().context(), "/assets/img/box_tex.png", tex_coords)?;

     let cube = scene.object_from_mesh(cube_geometry.clone(), cube_tex);
     let cube2 = scene.object_from_mesh(cube_geometry.clone(), Material::vertex_colors(colors));

     cube.set_position([10.,0.,10.]);
     scene.add(&cube);
     //scene.add(&cube2);

    let pan_gizmo = create_transform_gizmo(&scene, ArrowType::Sphere);
    scene.add(&pan_gizmo);
    let pan_gizmo = rc_rcell(pan_gizmo);
    
    let (a_sun, a_earth) = {
        //let scene = a_scene.borrow();
        let renderer = a_rndr.borrow();
        //let viewport = a_view.borrow();

        let mut sun = scene.object_from_mesh_and_name(
            Geometry::from_genmesh(&IcoSphere::subdivide(1)),
            Material::single_color(1.0, 1.0, 0.0, 1.0),
            "Sun",
        );

        let mut earth = scene.object_from_mesh_and_name(
            Geometry::from_genmesh(&IcoSphere::subdivide(1)),
            Material::single_color(0.0, 0.0, 1.0, 1.0),
            "Earth",
        );

        let moon = scene.object_from_mesh_and_name(
            Geometry::from_genmesh(&IcoSphere::subdivide(1)),
            Material::single_color(1.0, 1.0, 1.0, 1.0),
            "Moon",
        );

        moon.set_position([5.0, 0.0, 0.0]);
        earth.set_position([5.0, 0.0, 0.0]);
        moon.scale(0.5);
        earth.scale(0.5);
        sun.scale(2.0);
        sun.set_position([0.,-10.,0.]);

        let moon = rc_rcell(moon);
        earth.add(moon);
        let earth = rc_rcell(earth);
        sun.add(earth.clone());
        scene.add(&sun);
        renderer.setup_renderer();
        renderer.update_viewport(&viewport);
        (rc_rcell(sun), earth)
    };

    let a_cube = rc_rcell(cube);
    let a_scene = rc_rcell(scene);
    let a_view = rc_rcell(viewport);
    let editor = rc_rcell(Editor::new(a_view.clone(), a_scene.clone(), a_rndr.clone()));

    let p_g = pan_gizmo.clone();

    let f = rc_rcell(None);
    let g = f.clone();

    let b_rndr = a_rndr.clone();
    let b_view = a_view.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let mut renderer = b_rndr.borrow_mut();
        let view = b_view.borrow();
        let sun = a_sun.borrow();
        let earth = a_earth.borrow();
        earth.rotate_by(UnitQuaternion::from_euler_angles(0., 0.02, 0.));
        sun.rotate_by(UnitQuaternion::from_euler_angles(0., 0.01, 0.));
        let cube = a_cube.borrow_mut();
        cube.rotate_by(UnitQuaternion::from_euler_angles(0.01, 0.02, 0.));
        renderer.render(&a_scene.borrow());
        renderer.update_viewport(&view);
        let pan_gizmo = p_g.borrow_mut();
        match view.projection_type() {
            ProjectionType::Orthographic => {
                pan_gizmo.set_position(view.screen_to_world([-0.9,0.9,0.]));
                pan_gizmo.scale(0.14);
            }, ProjectionType::Perspective => {
                pan_gizmo.set_position(view.screen_to_world([-0.9,0.9,-0.4]));
                pan_gizmo.scale(0.0015);
            }
        }
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
    });
    
    let b_view = a_view.clone();
    let b_rndr = a_rndr.clone();
    add_event(&b_rndr.borrow().canvas(), "wheel", move |e| {
        let mut view = b_view.borrow_mut();
        let we = e.dyn_into::<WheelEvent>().unwrap();
        let dy = if we.delta_y() > 0. { 0.1 } else { -0.1 };
        view.enable_zoom();
        view.update_zoom(dy);
        view.disable_zoom();
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
            if me.button() == MouseButton::MIDDLE as i16 {
                view.enable_zoom();
            }
        });
        let b_view = a_view.clone();
        let b_rndr = a_rndr.clone();
        add_event(&window, "mouseup", move |e| {
            let mut view = b_view.borrow_mut();
            let renderer = b_rndr.borrow_mut();
            let me = e.dyn_into::<MouseEvent>().unwrap();
            let pressed_btn = me.button();
            if (pressed_btn == button as i16) || (pressed_btn == MouseButton::MIDDLE as i16) {
                renderer.change_cursor(CursorType::Pointer);
                view.disable_rotation();
                view.disable_zoom()
            }
        });
    }

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
