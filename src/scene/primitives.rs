use crate::{
    mesh::{Geometry, Material},
    node,
    renderer::RenderFlags,
    scene::{LightType, Node, Scene},
    Mesh,
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
    color: [f32; 4],
    arrow_type: ArrowTip,
    name: &str,
    has_stem: bool,
    depthless: bool,
) -> Node {
    let mut node = node!(scene, None, String::from(name));
    if has_stem {
        let stem = node!(
            scene,
            Some(Mesh::new(
                Geometry::from_genmesh_no_normals(&Cylinder::subdivide(8, 1)),
                Material::new_color_no_shade(color[0], color[1], color[2], color[3]),
            )),
            "Arrow Stem"
        );
        if depthless {
            let mut info = stem.info();
            info.render_flags = RenderFlags::no_depth();
            stem.set_info(info);
        }
        stem.set_scale_vec(0.2, 0.2, 5.);
        node.own(stem);
    }
    let head_geo = match arrow_type {
        ArrowTip::Cone => Some(Geometry::from_genmesh(&Cone::new(8))),
        ArrowTip::Cube => Some(Geometry::from_genmesh_no_normals(&Cube::new())),
        ArrowTip::Sphere => Some(Geometry::from_genmesh_no_normals(&IcoSphere::subdivide(1))),
        ArrowTip::None => None,
    };
    if let Some(head_geo) = head_geo {
        let head = node!(
            scene,
            Some(Mesh::new(
                head_geo,
                Material::new_color_no_shade(color[0], color[1], color[2], color[3]),
            )),
            "Arrow Head"
        );
        head.set_scale(0.8);
        head.set_position(0., 0., 5.);
        if depthless {
            let mut info = head.info();
            info.render_flags = RenderFlags::no_depth();
            head.set_info(info);
        }
        node.own(head);
    }
    node
}

pub fn create_light_node(scene: &Scene, light_type: LightType, color: [f32; 3]) -> Node {
    match light_type {
        LightType::Ambient => node!(
            scene,
            Some(Mesh::new(
                Geometry::from_genmesh_no_normals(&IcoSphere::new()),
                Material::new_wire(color[0], color[1], color[2], 1.),
            )),
            light_type.to_string(),
            RenderFlags::blend_cull(),
            DrawMode::Arrays
        ),
        LightType::Point => {
            let p = node!(
                scene,
                Some(Mesh::new(
                    Geometry::from_genmesh_no_normals(&IcoSphere::subdivide(2)),
                    Material::new_color_no_shade(color[0], color[1], color[2], 1.),
                )),
                light_type.to_string(),
                DrawMode::Triangle
            );
            p.set_scale(0.5);
            p
        }
        LightType::Spot => node!(
            scene,
            Some(Mesh::new(
                Geometry::from_genmesh_no_normals(&Cone::new(8)),
                Material::new_wire(color[0], color[1], color[2], 1.),
            )),
            light_type.to_string(),
            RenderFlags::blend_cull(),
            DrawMode::Arrays
        ),
        LightType::Directional => {
            let mut n = node!(scene, None, light_type.to_string());
            let cube = node!(
                scene,
                Some(Mesh::new(
                    Geometry::from_genmesh_no_normals(&Cube::new()),
                    Material::new_color_no_shade(color[0], color[1], color[2], 1.),
                )),
                "Plane",
                DrawMode::Triangle
            );
            cube.set_scale_vec(0.5, 0.5, 0.05);
            cube.rotate_by(UnitQuaternion::from_euler_angles(0., 0., PI / 4.));
            cube.set_position(0., 0., -1.);
            n.own(cube);
            for i in 0..5 {
                let ray = create_arrow(scene, [0.8, 0.8, 0.8, 1.], ArrowTip::Cone, "Ray", true, false);
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
                n.own(ray);
            }
            n
        }
    }
}

pub fn create_transform_gizmo(scene: &Scene, arrow_type: ArrowTip) -> Node {
    let name = match arrow_type {
        ArrowTip::Cone => "Translation",
        ArrowTip::Sphere => "Look",
        ArrowTip::Cube => "Scale",
        ArrowTip::None => "",
    };
    let x = create_arrow(scene, [0.8, 0., 0., 1.], arrow_type, "XAxis", true, true);
    let y = create_arrow(scene, [0., 0.8, 0., 1.], arrow_type, "YAxis", true, true);
    let z = create_arrow(scene, [0., 0., 0.8, 1.], arrow_type, "ZAxis", true, true);
    let mut node = node!(
        scene,
        Some(Mesh::new(
            Geometry::from_genmesh(&IcoSphere::subdivide(2)),
            Material::new_color_no_shade(0.8, 0.8, 0.8, 0.8),
        )),
        name,
        RenderFlags::no_depth()
    );
    let x_p = node!(
        scene,
        Some(Mesh::new(
            Geometry::from_genmesh(&Cube::new()),
            Material::new_color_no_shade(0.8, 0., 0., 1.),
        )),
        "XPlane",
        RenderFlags::no_depth()
    );
    let y_p = node!(
        scene,
        Some(Mesh::new(
            Geometry::from_genmesh(&Cube::new()),
            Material::new_color_no_shade(0., 0.8, 0., 1.),
        )),
        "YPlane",
        RenderFlags::no_depth()
    );
    let z_p = node!(
        scene,
        Some(Mesh::new(
            Geometry::from_genmesh(&Cube::new()),
            Material::new_color_no_shade(0., 0., 0.8, 1.),
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
    node.own(x);
    node.own(y);
    node.own(z);
    node.own(x_p);
    node.own(y_p);
    node.own(z_p);
    if arrow_type == ArrowTip::Sphere {
        let n_x = create_arrow(
            scene,
            [1.0, 0.0, 0.0, 1.0],
            arrow_type,
            "snap-x",
            false,
            true,
        );
        let n_y = create_arrow(
            scene,
            [0.0, 1.0, 0.0, 1.0],
            arrow_type,
            "snap-y",
            false,
            true,
        );
        let n_z = create_arrow(
            scene,
            [0.0, 0.0, 1.0, 1.0],
            arrow_type,
            "snap-z",
            false,
            true,
        );
        n_x.set_position(6., 0., 0.);
        n_y.set_position(0., -6., 0.);
        n_z.set_position(0., 0., -6.);
        node.own(n_x);
        node.own(n_y);
        node.own(n_z);
    }
    node
}

pub fn create_origin(scene: &Scene) -> Node {
    let x = create_arrow(scene, [1., 0., 0., 1.], ArrowTip::None, "XAxis", true, true);
    let y = create_arrow(scene, [0., 1., 0., 1.], ArrowTip::None, "YAxis", true, true);
    let z = create_arrow(scene, [0., 0., 1., 1.], ArrowTip::None, "ZAxis", true, true);
    x.rotate_by(UnitQuaternion::from_euler_angles(0.0, PI / 2., 0.0));
    y.rotate_by(UnitQuaternion::from_euler_angles(-PI / 2., 0.0, 0.0));
    z.rotate_by(UnitQuaternion::from_euler_angles(0.0, 0.0, PI / 2.));
    x.set_scale(0.5);
    y.set_scale(0.5);
    z.set_scale(0.5);
    let mut center = node!(
        scene,
        Some(Mesh::new(
            Geometry::from_genmesh_no_normals(&IcoSphere::subdivide(2)),
            Material::new_color_no_shade(1., 1., 1., 1.0),
        )),
        "Spawn Origin",
        RenderFlags::no_depth()
    );
    center.own(x);
    center.own(y);
    center.own(z);
    center
}

pub fn create_primitive_node(scene: &Scene, primitive: Primitive) -> Node {
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
        _ => node!(
            scene,
            Some(Mesh::new(geo, Material::new_color(1.0, 1.0, 1.0, 1.0),)),
            primitive.to_string()
        ),
    }
}
