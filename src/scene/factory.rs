use crate::{
    mesh::{Geometry,Material},
    scene::{Node,Scene},
};
use genmesh::generators::{Cylinder, Cone, Cube, IcoSphere, Torus};
use nalgebra::UnitQuaternion;
use std::f32::consts::PI;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ArrowType {
    Cone,
    Cube,
    Sphere,
}

pub fn create_arrow(scene: &Scene, color: [f32;4], arrow_type: ArrowType, has_stem: bool) -> Node {
    let mut node = scene.empty();
    if has_stem {
        let stem = scene.object_from_mesh_and_name(
            Geometry::from_genmesh_no_normals(&Cylinder::subdivide(8, 1)),
            Material::single_color_no_shade(color[0], color[1], color[2], color[3]),
            "Arrow Stem"
        );
        stem.scale_vec([0.2, 3., 0.2]);
        node.add(stem);
    }
    let head = match arrow_type {
        ArrowType::Cone => {
            let head = scene.object_from_mesh_and_name(
                Geometry::from_genmesh(&Cone::new(8)),
                Material::single_color_no_shade(color[0], color[1], color[2], color[3]),
                "Pointy Arrow Head");
            head.scale(0.5);
            head

        }, ArrowType::Cube => {
            let head = scene.object_from_mesh_and_name(
                Geometry::from_genmesh_no_normals(&Cube::new()),
                Material::single_color_no_shade(color[0], color[1], color[2], color[3]),
                "Flat Arrow Head");
            head.scale(0.4);
            head
        }, ArrowType::Sphere => {
            let head = scene.object_from_mesh_and_name(
                Geometry::from_genmesh_no_normals(&IcoSphere::subdivide(1)),
                Material::single_color_no_shade(color[0], color[1], color[2], color[3]),
                "Flat Arrow Head"
            );
            head.scale(0.8);
            head
        },
    };
    head.set_position([0.,0.,3.]);
    node.add(head);
    node
}

pub fn create_transform_gizmo(scene: &Scene, arrow_type: ArrowType) -> Node {
    let mut node = scene.object_from_mesh_and_name(Geometry::from_genmesh(&IcoSphere::subdivide(1)), Material::single_color_no_shade(1.0,1.0,1.0,1.0), "Gizmo");
    node.scale(0.5);
    let x = create_arrow(scene, [1.0,0.0,0.0,1.0], arrow_type, true);
    let y = create_arrow(scene, [0.0,1.0,0.0,1.0], arrow_type, true);
    let z = create_arrow(scene, [0.0,0.0,1.0,1.0], arrow_type, true);
    x.set_position([0.,0.,3.]);
    y.set_position([0.,0.,3.]);
    z.set_position([0.,0.,3.]);
    x.rotate_by(UnitQuaternion::from_euler_angles(0.0,PI/2.,0.0));
    y.rotate_by(UnitQuaternion::from_euler_angles(-PI/2.,0.0,0.0));
    z.rotate_by(UnitQuaternion::from_euler_angles(0.0,0.0,PI/2.));
    node.add(x);
    node.add(y);
    node.add(z);
    if arrow_type == ArrowType::Sphere {
        let n_x = create_arrow(scene, [1.0,0.0,0.0,1.0], arrow_type, false);
        let n_y = create_arrow(scene, [0.0,1.0,0.0,1.0], arrow_type, false);
        let n_z = create_arrow(scene, [0.0,0.0,1.0,1.0], arrow_type, false);
        n_x.set_position([0.,0.,-9.]);
        n_y.set_position([0.,0.,-9.]);
        n_z.set_position([0.,0.,-9.]);
        n_x.rotate_by(UnitQuaternion::from_euler_angles(0.0,PI/2.,0.0));
        n_y.rotate_by(UnitQuaternion::from_euler_angles(-PI/2.,0.0,0.0));
        n_z.rotate_by(UnitQuaternion::from_euler_angles(0.0,0.0,PI/2.));
        node.add(n_x);
        node.add(n_y);
        node.add(n_z);
    }
    node
}
