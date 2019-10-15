use crate::{
    mesh::{Geometry, Material},
    scene::{Node, Scene, LightType},
    renderer::DrawMode,
};
use genmesh::generators::{Circle, Cone, Cube, Cylinder, IcoSphere, Plane, SphereUv, Torus};
use nalgebra::{UnitQuaternion, Isometry3};
use std::f32::consts::PI;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ArrowType {
    Cone,
    Cube,
    Sphere,
}

/// Various primitive types (eg. Plane, Cube, Torus, IcoSphere, etc).
#[derive(Copy, Clone, Debug, PartialEq)]
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

impl From<&str> for Primitive {
    fn from(t: &str) -> Primitive {
        match t {
            "Plane" => Primitive::Plane,
            "Cube" => Primitive::Cube,
            "Circle" => Primitive::Circle,
            "IcoSphere" => Primitive::IcoSphere,
            "Cylinder" => Primitive::Cylinder,
            "Cone" => Primitive::Cone,
            "UVSphere" => Primitive::UVSphere,
            "Torus" => Primitive::Torus,
            _ => Primitive::Empty,
        }
    }
}

use std::fmt;

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
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

#[derive(Clone, Debug, PartialEq)]
pub enum Gizmo {
    Translate(Node, GizmoGrab, Isometry3<f32>),
    Scale(Node, GizmoGrab, Isometry3<f32>),
    Rotate(Node, GizmoGrab, Isometry3<f32>),
}

impl Gizmo {
    pub fn inner(&self) -> (&Node, &GizmoGrab, &Isometry3<f32>) {
        match self {
            Gizmo::Translate(n, s, i) => (n, s, i),
            Gizmo::Scale(n, s, i) => (n, s, i),
            Gizmo::Rotate(n, s, i) => (n, s, i),
        }
    }
    pub fn inner_mut(&mut self) -> (&Node, &mut GizmoGrab, &mut Isometry3<f32>) {
        match self {
            Gizmo::Translate(n, s, i) => (n, s, i),
            Gizmo::Scale(n, s, i) => (n, s, i),
            Gizmo::Rotate(n, s, i) => (n, s, i),
        }
    }
}

pub fn create_arrow(scene: &Scene, color: [f32; 4], arrow_type: ArrowType, name: &str, has_stem: bool) -> Node {
    let mut node = scene.empty_w_name(name);
    if has_stem {
        let stem = scene.object_from_mesh_name_and_mode(
            Geometry::from_genmesh_no_normals(&Cylinder::subdivide(8, 1)),
            Material::single_color_no_shade(color[0], color[1], color[2], color[3]),
            "Arrow Stem",
            DrawMode::TriangleNoDepth
        );
        stem.set_scale_vec(0.2, 0.2, 3.);
        node.own(stem);
    }
    let head = match arrow_type {
        ArrowType::Cone => {
            let head = scene.object_from_mesh_name_and_mode(
                Geometry::from_genmesh(&Cone::new(8)),
                Material::single_color_no_shade(color[0], color[1], color[2], color[3]),
                "Arrow Head",
                DrawMode::TriangleNoDepth
            );
            head.set_scale(0.5);
            head
        }
        ArrowType::Cube => {
            let head = scene.object_from_mesh_name_and_mode(
                Geometry::from_genmesh_no_normals(&Cube::new()),
                Material::single_color_no_shade(color[0], color[1], color[2], color[3]),
                "Arrow Head",
                DrawMode::TriangleNoDepth
            );
            head.set_scale(0.4);
            head
        }
        ArrowType::Sphere => {
            let head = scene.object_from_mesh_name_and_mode(
                Geometry::from_genmesh_no_normals(&IcoSphere::subdivide(1)),
                Material::single_color_no_shade(color[0], color[1], color[2], color[3]),
                "Arrow Head",
                DrawMode::TriangleNoDepth
            );
            head.set_scale(0.8);
            head
        }
    };
    if has_stem {
        head.set_position(0., 0., 3.);
    }
    node.own(head);
    node
}

pub fn create_light_node(scene: &Scene, light_type: LightType, color: [f32;3]) -> Node {
    match light_type {
        LightType::Ambient => scene.object_from_mesh_name_and_mode(
            Geometry::from_genmesh_no_normals(&IcoSphere::new()),
            Material::wireframe(color[0],color[1],color[2],1.),
            &light_type.to_string(),
            DrawMode::Wireframe,
        ),
        LightType::Point => {
            let p = scene.object_from_mesh_name_and_mode(
                Geometry::from_genmesh_no_normals(&IcoSphere::subdivide(2)),
                Material::single_color_no_shade(color[0],color[1],color[2],1.),
                &light_type.to_string(),
                DrawMode::Triangle,
            );
            p.set_scale(0.5);
            p
        }
        LightType::Spot => scene.object_from_mesh_name_and_mode(
            Geometry::from_genmesh_no_normals(&Cone::new(8)),
            Material::wireframe(color[0],color[1],color[2],1.),
            &light_type.to_string(),
            DrawMode::Wireframe,
        ),
        LightType::Directional => {
            let mut n = scene.empty_w_name("Directional");
            let cube = scene.object_from_mesh_name_and_mode(
                Geometry::from_genmesh_no_normals(&Cube::new()),
                Material::single_color_no_shade(color[0],color[1],color[2],1.),
                &light_type.to_string(),
                DrawMode::Triangle,
            );
            cube.set_scale_vec(0.5, 0.5, 0.05);
            cube.rotate_by(UnitQuaternion::from_euler_angles(0., 0., PI / 4.));
            cube.set_position(0., 0., -1.);
            n.own(cube);
            for i in 0..5 {
                let ray = create_arrow(scene, [1.,1.,0.,1.], ArrowType::Cone, "sun_ray", true);
                ray.set_scale(0.25);
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

pub fn create_transform_gizmo(scene: &Scene, arrow_type: ArrowType) -> Node {
    let name = match arrow_type {
        ArrowType::Cone => "translation",
        ArrowType::Sphere => "look",
        ArrowType::Cube => "scale",
    };
    let x = create_arrow(scene, [0.8, 0., 0., 1.], arrow_type, "x-axis", true);
    let y = create_arrow(scene, [0., 0.8, 0., 1.], arrow_type, "y-axis", true);
    let z = create_arrow(scene, [0., 0., 0.8, 1.], arrow_type, "z-axis", true);
    let mut node = scene.object_from_mesh_name_and_mode(
        Geometry::from_genmesh(&IcoSphere::subdivide(2)),
        Material::single_color_no_shade(0.8, 0.8, 0.8, 0.8),
        name,
        DrawMode::TriangleNoDepth
    );
    let x_p = scene.object_from_mesh_name_and_mode(
        Geometry::from_genmesh(&Cube::new()),
        Material::single_color_no_shade(0.8, 0., 0., 1.),
        "pan_x",
        DrawMode::TriangleNoDepth
    );
    let y_p = scene.object_from_mesh_name_and_mode(
        Geometry::from_genmesh(&Cube::new()),
        Material::single_color_no_shade(0., 0.8, 0., 1.),
        "pan_y",
        DrawMode::TriangleNoDepth
    );
    let z_p = scene.object_from_mesh_name_and_mode(
        Geometry::from_genmesh(&Cube::new()),
        Material::single_color_no_shade(0., 0., 0.8, 1.),
        "pan_z",
        DrawMode::TriangleNoDepth
    );
    x.set_position(3., 0., 0.);
    y.set_position(0., 3., 0.);
    z.set_position(0., 0., 3.);
    x.rotate_by(UnitQuaternion::from_euler_angles(0.0, PI / 2., 0.0));
    y.rotate_by(UnitQuaternion::from_euler_angles(-PI / 2., 0.0, 0.0));
    z.rotate_by(UnitQuaternion::from_euler_angles(0.0, 0.0, PI / 2.));
    x_p.set_position(0., 3., 3.);
    y_p.set_position(3., 0., 3.);
    z_p.set_position(3., 3., 0.);
    x_p.set_scale_vec(0.2,1.,1.);
    y_p.set_scale_vec(1.,0.2,1.);
    z_p.set_scale_vec(1.,1.,0.2);
    node.own(x);
    node.own(y);
    node.own(z);
    node.own(x_p);
    node.own(y_p);
    node.own(z_p);
    if arrow_type == ArrowType::Sphere {
        let n_x = create_arrow(scene, [1.0, 0.0, 0.0, 1.0], arrow_type, "snap-x", false);
        let n_y = create_arrow(scene, [0.0, 1.0, 0.0, 1.0], arrow_type, "snap-y", false);
        let n_z = create_arrow(scene, [0.0, 0.0, 1.0, 1.0], arrow_type, "snap-z", false);
        n_x.set_position(6., 0., 0.);
        n_y.set_position(0., -6., 0.);
        n_z.set_position(0., 0., -6.);
        node.own(n_x);
        node.own(n_y);
        node.own(n_z);
    }
    node
}


pub fn create_primitive_node(scene: &Scene, primitive: Primitive) -> Node {
    match primitive {
        Primitive::Plane => scene.object_from_mesh_and_name(
            Geometry::from_genmesh(&Plane::new()),
            Material::single_color(1.0, 1.0, 1.0, 1.0),
            &primitive.to_string(),
        ),
        Primitive::IcoSphere => scene.object_from_mesh_and_name(
            Geometry::from_genmesh(&IcoSphere::new()),
            Material::single_color(1.0, 1.0, 1.0, 1.0),
            &primitive.to_string(),
        ),
        Primitive::Cube => scene.object_from_mesh_and_name(
            Geometry::from_genmesh(&Cube::new()),
            Material::single_color(1.0, 1.0, 1.0, 1.0),
            &primitive.to_string(),
        ),
        Primitive::Circle => scene.object_from_mesh_and_name(
            Geometry::from_genmesh(&Circle::new(8)),
            Material::single_color(1.0, 1.0, 1.0, 1.0),
            &primitive.to_string(),
        ),
        Primitive::Cylinder => scene.object_from_mesh_and_name(
            Geometry::from_genmesh(&Cylinder::new(8)),
            Material::single_color(1.0, 1.0, 1.0, 1.0),
            &primitive.to_string(),
        ),
        Primitive::Cone => scene.object_from_mesh_and_name(
            Geometry::from_genmesh(&Cone::new(8)),
            Material::single_color(1.0, 1.0, 1.0, 1.0),
            &primitive.to_string(),
        ),
        Primitive::UVSphere => scene.object_from_mesh_and_name(
            Geometry::from_genmesh(&SphereUv::new(8, 16)),
            Material::single_color(1.0, 1.0, 1.0, 1.0),
            &primitive.to_string(),
        ),
        Primitive::Torus => scene.object_from_mesh_and_name(
            Geometry::from_genmesh(&Torus::new(1., 0.2, 16, 8)),
            Material::single_color(1.0, 1.0, 1.0, 1.0),
            &primitive.to_string(),
        ),
        Primitive::Empty => scene.empty(),
    }
}
