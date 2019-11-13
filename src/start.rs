use crate::{
    controller::ProjectionConfig,
    dom_factory::{document, loop_animation_frame},
    editor::{console_setup, ConsoleConfig},
    rc_rcell,
    renderer::{Renderer, RendererConfig},
    scene::LightType,
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
    let cube_tex = Material::new_texture("/assets/img/box_tex.png", tex_coords)
        .expect("Couldn't load texture");
    let cube_mesh = Mesh::new(cube_geometry.clone(), cube_tex);
    let mut cube = node!(scene, Some(cube_mesh), "Wooden Cube");
    let mesh = Mesh::new(cube_geometry, Material::vertex_colors(colors));
    let cube2 = node!(scene, Some(mesh), "Colored Cube");
    cube.set_position(5., 0., 5.);
    cube.set_scale(0.2);
    cube2.set_position(4., 0., 0.);
    let a_cube2 = rc_rcell(cube2);
    //cube.add(a_cube2);
    cube
}

fn gen_sphere_uv(geometry: &Geometry) -> Vec<f32> {
    let mut uvs = Vec::new();
    for each in geometry.vertices.chunks(3) {
        uvs.push(0.5 + f32::atan2(each[1], each[0]) / (2. * PI));
        uvs.push(0.5 - f32::asin(each[2]) / PI);
    }
    uvs
}

/// The main entrypoint that is automatically executed on page load.
#[wasm_bindgen(start)]
#[allow(dead_code)]
pub fn start() -> Result<(), JsValue> {
    document().set_title("Editor | Moksha");
    console_setup(ConsoleConfig {
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

    let (sun, _earth, _moon) = {
        //let scene = a_scene.borrow();
        let renderer = a_rndr.borrow();
        //let viewport = a_view.borrow();

        let mut sun = node!(
            scene,
            Some(Mesh::new(
                Geometry::from_genmesh_no_normals(&IcoSphere::subdivide(2)),
                Material::new_color_no_shade(1.0, 1.0, 0.0, 1.0),
            )),
            "Sun"
        );

        let mut earth = node!(
            scene,
            Some(Mesh::new(
                Geometry::from_genmesh(&SphereUv::new(16, 8)),
                Material::new_color(0.0, 0.0, 1.0, 1.0).wire_overlay(),
            )),
            "Earth"
        );
        //let geo = Geometry::from_genmesh(&SphereUv::new(8, 4));
        //let tex = Material::new_texture("/assets/img/earth.jpg", gen_sphere_uv(&geo))
        //.expect("Couldn't load texture");
        //let mesh = Mesh::new(geo, tex);
        //let mut earth = node!(scene, mesh, "Earth");
        //let mut earth = scene.object_from_obj(
        //include_str!("assets/obj/earth.obj"),
        //"/assets/img/earth.jpg",
        //);
        let mut moon = node!(
            scene,
            Some(Mesh::new(
                Geometry::from_genmesh(&IcoSphere::subdivide(2)),
                Material::new_color(1.0, 1.0, 1.0, 1.0),
            )),
            "Moon"
        );

        let cube = rc_rcell(cube(&scene, &a_rndr.borrow()));
        moon.add(cube);
        moon.set_position(6.0, 0.0, 0.0);
        earth.set_position(10.0, 0.0, 0.0);
        earth.set_scale(0.5);
        moon.set_scale(0.5);
        sun.set_scale(2.0);

        let moon = rc_rcell(moon);
        earth.add(moon.clone());
        let earth = rc_rcell(earth);
        sun.add(earth.clone());
        let sun = rc_rcell(sun);
        (sun, earth, moon)
    };

    let ambient = scene.light(LightType::Ambient, [1.0, 1.0, 1.0], 0.1);
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

    let directional = scene.light(LightType::Directional, [1., 1., 1.], 2.0);
    let dir_node = directional.node();
    dir_node.borrow().set_position(30., 0., -10.);

    scene.add(sun.clone());
    scene.add_light(&ambient);
    ////scene.add_light(point2);
    ////scene.add_light(point);
    scene.add_light(&directional);
    //scene.add_light(spot);

    //_earth.borrow().rotate_by(UnitQuaternion::from_euler_angles(0., 0.02, 0.));

    let a_scene = Rc::new(scene);
    let mut editor = Editor::new(a_scene.clone());
    editor.set_active_node(_earth);
    let a_editor = rc_rcell(editor);
    sun.borrow()
        .rotate_by(UnitQuaternion::from_euler_angles(0., PI / 3., 0.));
    loop_animation_frame(move || {
        {
            //a_earth.borrow().rotate_by(UnitQuaternion::from_euler_angles(0., 0.02, 0.));
            //sun.borrow().rotate_by(UnitQuaternion::from_euler_angles(0., 0.01, 0.));
        }
        {
            a_editor.borrow().track_gizmo();
            a_rndr.borrow_mut().render(&a_scene, &a_view.borrow());
        }
    });
    Ok(())
}
