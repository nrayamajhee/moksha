use super::{object::Object, LightType, Scene, SceneObject};
use crate::{
    Color,
    mesh::{Geometry, Material, Mesh},
    object,
    renderer::RenderFlags,
};
use genmesh::generators::{Circle, Cone, Cube, Cylinder, IcoSphere, Plane, SphereUv, Torus};
use nalgebra::UnitQuaternion;
use std::f32::consts::PI;
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ArrowTip {
    Cone,
    Cube,
    Sphere,
    None,
}

/// Various primitive types (eg. Plane, Cube, Torus, IcoSphere, etc).

#[derive(Copy, Clone, Debug, PartialEq, Eq, Display, EnumIter, EnumString)]
pub enum Primitive {
    Plane,
    Cube,
    Circle,
    IcoSphere,
    Cylinder,
    Cone,
    UVSphere,
    Torus,
    Empty,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GizmoGrab {
    XAxis,
    YAxis,
    ZAxis,
    XPlane,
    YPlane,
    ZPlane,
    ViewPlane,
    None,
}

pub fn create_arrow(
    scene: &Scene,
    color: Color,
    arrow_type: ArrowTip,
    name: &str,
    has_stem: bool,
    depthless: bool,
) -> SceneObject {
    let object = object!(scene, None, String::from(name));
    if has_stem {
        let stem = object!(
            scene,
            Some(Mesh::new(
                Geometry::from_genmesh_no_normals(&Cylinder::subdivide(8, 1)),
                Material::new_color_no_shade(color)
            )),
            "Arrow Stem"
        );
        if depthless {
            let mut info = stem.info();
            info.render_flags = RenderFlags::no_depth();
            stem.set_info(info);
        }
        stem.set_scale_vec(0.2, 0.2, 5.);
        object.add(&stem);
    }
    let head_geo = match arrow_type {
        ArrowTip::Cone => Some(Geometry::from_genmesh(&Cone::new(8))),
        ArrowTip::Cube => Some(Geometry::from_genmesh_no_normals(&Cube::new())),
        ArrowTip::Sphere => Some(Geometry::from_genmesh_no_normals(&IcoSphere::subdivide(1))),
        ArrowTip::None => None,
    };
    if let Some(head_geo) = head_geo {
        let head = object!(
            scene,
            Some(Mesh::new(head_geo, Material::new_color_no_shade(color),)),
            "Arrow Head"
        );
        head.set_scale(0.8);
        head.set_position(0., 0., 5.);
        if depthless {
            let mut info = head.info();
            info.render_flags = RenderFlags::no_depth();
            head.set_info(info);
        }
        object.add(&head);
    }
    object
}

pub fn create_light(scene: &Scene, light_type: LightType, color: Color) -> SceneObject {
    match light_type {
        LightType::Ambient => object!(
            scene,
            Some(Mesh::new(
                Geometry::from_genmesh_no_normals(&IcoSphere::new()),
                Material::new_wire(color)
            )),
            light_type.to_string(),
            RenderFlags::blend_cull(),
            DrawMode::Arrays
        ),
        LightType::Point => {
            let p = object!(
                scene,
                Some(Mesh::new(
                    Geometry::from_genmesh_no_normals(&IcoSphere::subdivide(2)),
                    Material::new_wire(color)
                )),
                light_type.to_string(),
                DrawMode::Triangle
            );
            p.set_scale(0.5);
            p
        }
        LightType::Spot => object!(
            scene,
            Some(Mesh::new(
                Geometry::from_genmesh_no_normals(&Cone::new(8)),
                Material::new_wire(color)
            )),
            light_type.to_string(),
            RenderFlags::blend_cull(),
            DrawMode::Arrays
        ),
        LightType::Directional => {
            let n = object!(scene, None, light_type.to_string());
            let cube = object!(
                scene,
                Some(Mesh::new(
                    Geometry::from_genmesh_no_normals(&Cube::new()),
                    Material::new_color_no_shade(color)
                )),
                "Plane",
                DrawMode::Triangle
            );
            cube.set_scale_vec(0.5, 0.5, 0.05);
            cube.rotate_by(UnitQuaternion::from_euler_angles(0., 0., PI / 4.));
            cube.set_position(0., 0., -1.);
            n.add(&cube);
            for i in 0..5 {
                let ray = create_arrow(scene, Color::gray(0.8), ArrowTip::Cone, "Ray", true, false);
                ray.set_scale(0.1);
                match i % 5 {
                    0 => {
                        ray.set_position(0.5, 0., 0.);
                    }
                    1 => {
                        ray.set_position(-0.5, 0., 0.);
                    }
                    2 => {
                        ray.set_position(0., 0.5, 0.);
                    }
                    3 => {
                        ray.set_position(0., -0.5, 0.);
                    }
                    _ => (),
                }
                n.add(&ray);
            }
            n
        }
    }
}

pub fn create_transform_gizmo(scene: &Scene, arrow_type: ArrowTip) -> SceneObject {
    let name = match arrow_type {
        ArrowTip::Cone => "Translation",
        ArrowTip::Sphere => "Look",
        ArrowTip::Cube => "Scale",
        ArrowTip::None => "",
    };
    let x = create_arrow(scene, Color::redish(0.8), arrow_type, "XAxis", true, true);
    let y = create_arrow(scene, Color::greenish(0.8), arrow_type, "YAxis", true, true);
    let z = create_arrow(scene, Color::bluish(0.8), arrow_type, "ZAxis", true, true);
    let mut object = object!(
        scene,
        Some(Mesh::new(
            Geometry::from_genmesh(&IcoSphere::subdivide(2)),
            Material::new_color_no_shade(Color::rgb(0.8, 0.8, 0.8,)),
        )),
        name,
        RenderFlags::no_depth()
    );
    let x_p = object!(
        scene,
        Some(Mesh::new(
            Geometry::from_genmesh(&Cube::new()),
            Material::new_color_no_shade(Color::redish(0.8)),
        )),
        "XPlane",
        RenderFlags::no_depth()
    );
    let y_p = object!(
        scene,
        Some(Mesh::new(
            Geometry::from_genmesh(&Cube::new()),
            Material::new_color_no_shade(Color::greenish(0.8)),
        )),
        "YPlane",
        RenderFlags::no_depth()
    );
    let z_p = object!(
        scene,
        Some(Mesh::new(
            Geometry::from_genmesh(&Cube::new()),
            Material::new_color_no_shade(Color::bluish(0.8)),
        )),
        "ZPlane",
        RenderFlags::no_depth()
    );
    x.set_position(5., 0., 0.);
    y.set_position(0., 5., 0.);
    z.set_position(0., 0., 5.);
    x.rotate_by(UnitQuaternion::from_euler_angles(0.0, PI / 2., 0.0));
    y.rotate_by(UnitQuaternion::from_euler_angles(-PI / 2., 0.0, 0.0));
    z.rotate_by(UnitQuaternion::from_euler_angles(0.0, 0.0, PI / 2.));
    x_p.set_position(0., 5., 5.);
    y_p.set_position(5., 0., 5.);
    z_p.set_position(5., 5., 0.);
    x_p.set_scale_vec(0.2, 1., 1.);
    y_p.set_scale_vec(1., 0.2, 1.);
    z_p.set_scale_vec(1., 1., 0.2);
    object.add(&x);
    object.add(&y);
    object.add(&z);
    object.add(&x_p);
    object.add(&y_p);
    object.add(&z_p);
    if arrow_type == ArrowTip::Sphere {
        let n_x = create_arrow(scene, Color::red(), arrow_type, "snap-x", false, true);
        let n_y = create_arrow(scene, Color::green(), arrow_type, "snap-y", false, true);
        let n_z = create_arrow(scene, Color::blue(), arrow_type, "snap-z", false, true);
        n_x.set_position(6., 0., 0.);
        n_y.set_position(0., -6., 0.);
        n_z.set_position(0., 0., -6.);
        object.add(&n_x);
        object.add(&n_y);
        object.add(&n_z);
    }
    object
}

pub fn create_origin(scene: &Scene) -> SceneObject {
    let x = create_arrow(scene, Color::red(), ArrowTip::None, "XAxis", true, true);
    let y = create_arrow(scene, Color::green(), ArrowTip::None, "YAxis", true, true);
    let z = create_arrow(scene, Color::blue(), ArrowTip::None, "ZAxis", true, true);
    x.rotate_by(UnitQuaternion::from_euler_angles(0.0, PI / 2., 0.0));
    y.rotate_by(UnitQuaternion::from_euler_angles(-PI / 2., 0.0, 0.0));
    z.rotate_by(UnitQuaternion::from_euler_angles(0.0, 0.0, PI / 2.));
    x.set_scale(0.5);
    y.set_scale(0.5);
    z.set_scale(0.5);
    let mut center = object!(
        scene,
        Some(Mesh::new(
            Geometry::from_genmesh_no_normals(&IcoSphere::subdivide(2)),
            Material::new_color_no_shade(Color::white()),
        )),
        "Spawn Origin",
        RenderFlags::no_depth()
    );
    center.add(&x);
    center.add(&y);
    center.add(&z);
    center
}

pub fn create_primitive(scene: &Scene, primitive: Primitive) -> SceneObject {
    let geo = match primitive {
        Primitive::Plane => Geometry::from_genmesh(&Plane::new()),
        Primitive::IcoSphere => Geometry::from_genmesh(&IcoSphere::new()),
        Primitive::Cube => Geometry::from_genmesh(&Cube::new()),
        Primitive::Circle => Geometry::from_genmesh(&Circle::new(8)),
        Primitive::Cylinder => Geometry::from_genmesh(&Cylinder::new(8)),
        Primitive::Cone => Geometry::from_genmesh(&Cone::new(8)),
        Primitive::UVSphere => Geometry::from_genmesh(&SphereUv::new(8, 16)),
        Primitive::Torus => Geometry::from_genmesh(&Torus::new(1., 0.2, 16, 8)),
        Primitive::Empty => Geometry::default(),
    };
    match primitive {
        Primitive::Empty => scene.empty("Empty"),
        _ => object!(
            scene,
            Some(Mesh::new(geo, Material::new_color(Color::white()))),
            primitive.to_string()
        ),
    }
}
