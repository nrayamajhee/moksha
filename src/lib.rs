#![feature(proc_macro_hygiene)]
#![doc(
    html_logo_url = "https://moksha.rayamajhee.com/assets/img/icon.png",
    html_favicon_url = "https://moksha.rayamajhee.com/assets/img/icon.png"
)]

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[macro_use]
mod log_macros;

use std::cell::RefCell;
use std::rc::Rc;

/// Shorthand for Rc<RefCell\<T\>>.
pub type RcRcell<T> = Rc<RefCell<T>>;

/// Shorthand for Rc::new(RefCell::new(T)).
pub fn rc_rcell<T>(inner: T) -> RcRcell<T> {
    Rc::new(RefCell::new(inner))
}

use genmesh::generators::{IcoSphere, Cube};
use maud::html;
use std::f32::consts::PI;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{KeyboardEvent, MouseEvent, WheelEvent};

pub mod dom_factory;
use dom_factory::{add_event, body, document, request_animation_frame, window};

pub mod controller;
pub mod editor;
pub mod mesh;
pub mod renderer;
pub mod scene;

#[doc(inline)]
pub use crate::{
    controller::{MouseButton, ProjectionType, Viewport},
    editor::Editor,
    mesh::{Geometry, Material, Mesh, Transform},
    renderer::Renderer,
    scene::{Node, Primitive, Scene, Storage, ObjectInfo, Light, LightType},
};

use controller::ProjectionConfig;
use editor::console_setup;
use renderer::{CursorType};

/// The main entrypoint that is automatically executed on page load.
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
    let mut scene = Scene::new(a_rndr.clone());

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
        a_rndr.borrow().context(),
        "/assets/img/box_tex.png",
        tex_coords,
    )?;

    let mut cube = scene.object_from_mesh_and_name(cube_geometry.clone(), cube_tex, "Wooden Cube");
    let cube2 = scene.object_from_mesh_and_name(
        cube_geometry.clone(),
        Material::vertex_colors(colors),
        "Colored Cube",
    );

    cube.set_position(0., 1., 0.);
    cube.set_scale(0.2);
    cube2.set_position(4., 0., 0.);

    let a_cube2 = rc_rcell(cube2);
    cube.add(a_cube2);
    let a_cube = rc_rcell(cube);
    //scene.add(a_cube.clone());
    //scene.add(a_cube2.clone());

    let ambient_light = rc_rcell({
        let amb = scene.light(
            LightType::Ambient,
            1.0,
            [1.0,1.0,0.],
        );
        amb.set_position(10.,0.,-10.);
        amb
    });
    scene.add(ambient_light.clone());

    let (a_sun, a_earth, a_moon) = {
        //let scene = a_scene.borrow();
        let renderer = a_rndr.borrow();
        //let viewport = a_view.borrow();

        let mut sun = scene.object_from_mesh_and_name(
            Geometry::from_genmesh_no_normals(&IcoSphere::subdivide(2)),
            Material::single_color_no_shade(1.0, 1.0, 0.0, 1.0),
            "Sun",
        );

        let mut earth = scene.object_from_mesh_and_name(
            Geometry::from_genmesh(&IcoSphere::subdivide(2)),
            Material::single_color(0.0, 0.0, 1.0, 1.0),
            "Earth",
        );

        let moon = scene.object_from_mesh_and_name(
            Geometry::from_genmesh(&IcoSphere::subdivide(1)),
            Material::single_color(1.0, 1.0, 1.0, 1.0),
            "Moon",
        );

        moon.set_position(6.0, 0.0, 0.0);
        earth.set_position(10.0, 0.0, 0.0);
        earth.set_scale(0.5);
        moon.set_scale(0.5);
        sun.set_scale(2.0);

        let moon = rc_rcell(moon);
        earth.add(moon.clone());
        earth.add(a_cube.clone());
        let earth = rc_rcell(earth);
        sun.add(earth.clone());
        let sun = rc_rcell(sun);
        scene.add(sun.clone());
        renderer.setup_renderer();
        (sun, earth, moon)
    };

    let a_scene = rc_rcell(scene);
    let a_view = rc_rcell(viewport);
    let mut editor = Editor::new(a_view.clone(), a_scene.clone(), a_rndr.clone());
    editor.set_active_node(a_moon.clone());
    let a_editor = rc_rcell(editor);

    let f = rc_rcell(None);
    let g = f.clone();

    let b_rndr = a_rndr.clone();
    let b_view = a_view.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        {
            //a_earth.borrow().rotate_by(UnitQuaternion::from_euler_angles(0., 0.02, 0.));
            //a_sun.borrow().rotate_by(UnitQuaternion::from_euler_angles(0., 0.01, 0.));
        }
        {
            let mut view = b_view.borrow_mut();
            let mut editor = a_editor.borrow_mut();
            editor.update(&mut view);
            let renderer = b_rndr.borrow_mut();
            renderer.render(&a_scene.borrow(), &view);
        }
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    let b_rndr = a_rndr.clone();
    let b_view = a_view.clone();
    add_event(&window, "resize", move |_| {
        let mut renderer = b_rndr.borrow_mut();
        let mut viewport = b_view.borrow_mut();
        renderer.resize();
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
        view.enable_zoom();
        view.update_zoom(we.delta_y() as i32);
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
