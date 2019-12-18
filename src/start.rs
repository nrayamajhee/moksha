use crate::{
    controller::ProjectionConfig,
    dom_factory::{document, loop_animation_frame},
    editor::console::{self, ConsoleConfig},
    node, node_from_obj, node_from_obj_wired, rc_rcell,
    renderer::{Renderer, RendererConfig},
    scene::LightType,
    Color,
    Editor, Geometry, Material, Mesh, Node, Scene, Viewport,
};
use genmesh::generators::{Cube, IcoSphere, SphereUv};
use nalgebra::UnitQuaternion;
use std::f32::consts::PI;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

fn cube(scene: &Scene, renderer: &Renderer) -> Node {
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
    for face in face_colors {
        for each in face.iter() {
            for _ in 0..4 {
                colors.push(*each);
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
    let cube_tex = Material::new_texture("assets/img/box_tex.jpg", tex_coords);
    let cube_mesh = Mesh::new(cube_geometry.clone(), cube_tex);
    let mut cube = node!(scene, Some(cube_mesh), "Wooden Cube", DrawMode::Arrays);
    cube.set_position(5., 0., 5.);
    cube.set_scale(0.2);
    cube
}

/// The main entrypoint that is automatically executed on page load.
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    document().set_title("Editor | Moksha");
    console::setup(ConsoleConfig {
        ui_button: true,
        change_history: true,
    });
    let renderer = Renderer::new(RendererConfig {
        id: "gl-canvas",
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
    let scene = Scene::new(a_rndr.clone(), a_view.clone());
    scene.set_skybox("assets/img/milkyway", "jpg");
    let ambient = scene.light(LightType::Ambient, [1.0, 1.0, 1.0], 0.5);
    let amb_node = ambient.node();
    amb_node.borrow().set_position(10., 0., 10.);

    //let spot = scene.light(LightType::Spot, [1., 1., 1.], 1.0);
    //let spot_node = spot.node();
    //spot_node.borrow().set_position(25., 0., 10.);

    //let point = scene.light(LightType::Point, [1., 1., 1.], 1.0);
    //let point_node = point.node();
    //point_node.borrow().set_position(15., 0., 10.);

    //let point2 = scene.light(LightType::Point, [1., 1., 1.], 1.0);
    //let point2_node = point2.node();
    //point2_node.borrow().set_position(15., 0., -10.);

    let directional = scene.light(LightType::Directional, [1., 1., 1.], 1.0);
    let dir_node = directional.node();
    dir_node.borrow().set_position(30., 0., -10.);

    //scene.add(sun.clone());
    let cube = rc_rcell(cube(&scene, &a_rndr.borrow()));
    scene.add(cube.clone());
    scene.add_light(&ambient);
    ////scene.add_light(point2);
    ////scene.add_light(point);
    scene.add_light(&directional);

    let sphere = rc_rcell(node!(&scene,
    Some(Mesh::new(Geometry::from_genmesh(&IcoSphere::subdivide(2)),
    Material::new_color(0.5,0.5,0.5,1.0).wire_overlay())), "earth", DrawMode::Arrays));

    //let obj = rc_rcell(node_from_obj!(scene, "../assets/obj/earth", "earth"));
    //let obj = rc_rcell(node_from_obj_wired!(scene, "assets/obj/earth", "earth"));
    //obj.borrow().set_position(28., 0., 0.);
    //scene.add(obj.clone());
    scene.add(sphere.clone());
    let a_scene = Rc::new(scene);
    let mut editor = Editor::new(a_scene.clone());

    editor.set_active_node(sphere);
    let a_editor = rc_rcell(editor);
    //sun.borrow()
    //.rotate_by(UnitQuaternion::from_euler_angles(0., PI / 3., 0.));
    loop_animation_frame(move || {
        {
            //a_earth.borrow().rotate_by(UnitQuaternion::from_euler_angles(0., 0.02, 0.));
            //sun.borrow().rotate_by(UnitQuaternion::from_euler_angles(0., 0.01, 0.));
        }
        {
            //a_editor.borrow().track_gizmo();
            a_rndr.borrow_mut().render(&a_scene, &a_view.borrow());
        }
    }, Some(60.));
    Ok(())
}
