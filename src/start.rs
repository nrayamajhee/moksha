use crate::{
    controller::ProjectionConfig,
    dom_factory::{body, document, request_animation_frame},
    editor::{console_setup, ConsoleConfig},
    rc_rcell,
    renderer::{RenderConfig, Renderer},
    scene::LightType,
    Editor, Geometry, Material, Scene, Viewport,
};
use genmesh::generators::{Cube, IcoSphere};
use maud::html;
use std::f32::consts::PI;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

/// The main entrypoint that is automatically executed on page load.
#[wasm_bindgen(start)]
#[allow(dead_code)]
pub fn start() -> Result<(), JsValue> {
    console_setup(ConsoleConfig {
        ui_button: true,
        change_history: true,
    });
    let dom = html! {
        canvas #gl-canvas oncontextmenu="return false;" {}
    };
    document().set_title("Editor | Moksha");
    body()
        .insert_adjacent_html("beforeend", dom.into_string().as_str())
        .expect("Couldn't insert markup into the DOM!");
    let renderer = Renderer::new(RenderConfig {
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
    );

    let a_rndr = rc_rcell(renderer);
    let a_view = rc_rcell(viewport);
    let mut scene = Scene::new(a_rndr.clone(), a_view.clone());

    let cube = {
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
        let mut cube =
            scene.object_from_mesh_and_name(cube_geometry.clone(), cube_tex, "Wooden Cube");
        let cube2 = scene.object_from_mesh_and_name(
            cube_geometry.clone(),
            Material::vertex_colors(colors),
            "Colored Cube",
        );
        cube.set_position(5., 0., 5.);
        cube.set_scale(0.2);
        cube2.set_position(4., 0., 0.);
        let a_cube2 = Rc::new(cube2);
        cube.add(a_cube2);
        Rc::new(cube)
    };

    let (sun, _earth, _moon) = {
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
            Material::single_color_wired(0.0, 0.0, 1.0, 1.0),
            "Earth",
        );

        let moon = scene.object_from_mesh_and_name(
            Geometry::from_genmesh(&IcoSphere::subdivide(3)),
            Material::single_color_flat(1.0, 1.0, 1.0, 1.0),
            "Moon",
        );

        moon.set_position(6.0, 0.0, 0.0);
        earth.set_position(10.0, 0.0, 0.0);
        earth.set_scale(0.5);
        moon.set_scale(0.5);
        sun.set_scale(2.0);

        let moon = Rc::new(moon);
        earth.add(moon.clone());
        let earth = Rc::new(earth);
        sun.add(earth.clone());
        let sun = Rc::new(sun);
        renderer.setup_renderer();
        (sun, earth, moon)
    };

    let ambient = scene.light(LightType::Ambient, [0.1, 0.1, 0.1]);
    let amb_node = ambient.node();
    amb_node.set_position(10., 0., 10.);

    let spot = scene.light(LightType::Spot, [1., 1., 0.5]);
    let spot_node = spot.node();
    spot_node.set_position(25., 0., 10.);

    let point = scene.light(LightType::Point, [1., 0.5, 0.5]);
    let point_node = point.node();
    point_node.set_position(15., 0., 10.);

    let point2 = scene.light(LightType::Point, [0.5, 1., 0.5]);
    let point2_node = point2.node();
    point2_node.set_position(15., 0., -10.);

    let directional = scene.light(LightType::Directional, [0.5, 0.5, 1.]);
    let dir_node = directional.node();
    dir_node.set_position(30., 0., -10.);

    scene.add(cube.clone());
    scene.add(sun.clone());
    scene.add_light(ambient);
    //scene.add_light(point2);
    //scene.add_light(point);
    scene.add_light(directional);
    //scene.add_light(spot);

    let a_scene = rc_rcell(scene);
    let mut editor = Editor::new(a_view.clone(), a_scene.clone(), a_rndr.clone());
    editor.set_active_node(_moon.clone());
    let a_editor = rc_rcell(editor);

    let f = rc_rcell(None);
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        {
            //a_earth.borrow().rotate_by(UnitQuaternion::from_euler_angles(0., 0.02, 0.));
            //a_sun.borrow().rotate_by(UnitQuaternion::from_euler_angles(0., 0.01, 0.));
        }
        {
            let mut view = a_view.borrow_mut();
            a_editor.borrow_mut().update(&mut view);
            a_rndr.borrow_mut().render(&a_scene.borrow(), &view);
        }
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));
    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}
