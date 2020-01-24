use crate::{SceneObject, RcRcell, Viewport};
use nalgebra::{Isometry3, Vector3};
use ncollide3d::{
    query::Ray,
    query::RayCast,
    shape::{Ball, Plane as CollidePlane},
};
use std::str::FromStr;
use strum_macros::{Display, EnumIter, EnumString};
#[derive(Copy, Clone, Debug, PartialEq, Display, EnumIter, EnumString)]
pub enum CollisionConstraint {
    XAxis,
    YAxis,
    ZAxis,
    XPlane,
    YPlane,
    ZPlane,
    ViewPlane,
    None,
}

pub struct Gizmo {
    object: SceneObject,
    collision_constraint: CollisionConstraint,
    transform: Isometry3<f32>,
    offset: Vector3<f32>,
}

impl Gizmo {
    pub fn new(object: Object) -> Self {
        Self {
            object,
            collision_constraint: CollisionConstraint::None,
            transform: Isometry3::identity(),
            offset: Vector3::identity(),
        }
    }
    pub fn object(&self) -> &Object {
        &self.object
    }
    pub fn apply_target_transform(&self, target: &Object) {
        self.object.set_parent_transform(
            (target.parent_transform() * target.transform())
                .isometry
                .into(),
        );
        self.object
            .apply_parent_transform(self.object.parent_transform() * self.object.transform());
    }
    pub fn collision_constraint(&self) -> CollisionConstraint {
        self.collision_constraint
    }
    pub fn handle_mousedown(&mut self, ray: &Ray<f32>, view: &Viewport) -> bool {
        let p_t = self.object.parent_transform();
        let t = self.object.transform();
        let gizmo_node_t = (p_t * t).isometry;
        let target = Ball::new(t.scale.x);
        // if the central white ball is clicked
        if target.intersects_ray(&gizmo_node_t, ray) {
            self.transform = Isometry3::from_parts(
                gizmo_node_t.translation,
                view.isometry().inverse().rotation,
            );
            self.offset = Vector3::new(0., 0., 0.);
            self.collision_constraint = CollisionConstraint::ViewPlane;
            self.object().change_color([1., 1., 1.]);
            return true;
        }
        for child in self.object.owned_children() {
            let g_c = child.owned_children();
            // if the arrows are clicked
            let color = match child.info().name.as_str() {
                "XAxis" | "XPlane" => [1., 0., 0.],
                "YAxis" | "YPlane" => [0., 1., 0.],
                "ZAxis" | "ZPlane" => [0., 0., 1.],
                _ => [0., 0., 0.],
            };
            let constraint = CollisionConstraint::from_str(&child.info().name).unwrap();
            let t = if !g_c.is_empty() {
                let (tip, stem) = (&g_c[1], &g_c[0]);
                let (collided, transform) = if let Some(t) = tip.collides_w_ray(&ray) {
                    (true, t)
                } else if let Some(t) = stem.collides_w_ray(&ray) {
                    (true, t)
                } else {
                    (false, Isometry3::identity())
                };
                if collided {
                    stem.change_color(color);
                    tip.change_color(color);
                    Some(transform)
                } else {
                    None
                }
            } else {
                // if the cuboids are clicked
                if let Some(transform) = child.collides_w_ray(&ray) {
                    child.change_color(color);
                    Some(transform)
                } else {
                    None
                }
            };
            if let Some(transform) = t {
                self.collision_constraint = constraint;
                self.transform = transform;
                self.offset =
                    transform.translation.vector - Vector3::from(self.object.global_position());
                return true;
            }
        }
        false
    }
    pub fn handle_mousemove(&self, ray: &Ray<f32>, active_node: &Option<RcRcell<Object>>) {
        let collider = CollidePlane::new(match self.collision_constraint {
            CollisionConstraint::YAxis | CollisionConstraint::XPlane => Vector3::x_axis(),
            CollisionConstraint::XAxis
            | CollisionConstraint::ZAxis
            | CollisionConstraint::YPlane => Vector3::y_axis(),
            _ => Vector3::z_axis(),
        });
        if let Some(i) = collider.toi_and_normal_with_ray(&self.transform, &ray, false) {
            if let Some(object) = active_node.as_ref() {
                let object = object.borrow();
                let pos = object.position();
                // do calculation relative to parent element
                let poi = object
                    .parent_transform()
                    .inverse()
                    .transform_point(&ray.point_at(i.toi));
                let o = object
                    .parent_transform()
                    .inverse()
                    .transform_vector(&self.offset);
                let p = match self.collision_constraint {
                    CollisionConstraint::XAxis => [poi.x, pos[1] as f32, pos[2] as f32],
                    CollisionConstraint::YAxis => [pos[0] as f32, poi.y, pos[2] as f32],
                    CollisionConstraint::ZAxis => [pos[0] as f32, pos[1] as f32, poi.z],
                    _ => [poi.x, poi.y, poi.z],
                };
                object.set_position((p[0] - o.x).into(), (p[1] - o.y).into(), (p[2] - o.z).into());
                self.apply_target_transform(&object);
            }
        }
    }
    pub fn handle_mouseup(&mut self) {
        let color = match self.collision_constraint {
            CollisionConstraint::ViewPlane => [0.8, 0.8, 0.8],
            CollisionConstraint::XAxis | CollisionConstraint::XPlane => [0.8, 0., 0.],
            CollisionConstraint::YAxis | CollisionConstraint::YPlane => [0., 0.8, 0.],
            _ => [0., 0., 0.8],
        };
        if self.collision_constraint == CollisionConstraint::ViewPlane {
            self.object.change_color(color);
        } else {
            for child in self.object.owned_children() {
                let name = child.info().name;
                if name == self.collision_constraint.to_string() {
                    match name.as_str() {
                        "XAxis" | "YAxis" | "ZAxis" => {
                            for each in child.owned_children() {
                                each.change_color(color);
                            }
                        }
                        _ => {
                            child.change_color(color);
                        }
                    }
                    break;
                }
            }
        }
        self.collision_constraint = CollisionConstraint::None;
        self.transform = Isometry3::identity();
    }
}
