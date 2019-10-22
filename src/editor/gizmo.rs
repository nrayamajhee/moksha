use crate::{mesh::divide, Node, Viewport};
use nalgebra::{Isometry3, Vector3};
use ncollide3d::{
    query::Ray,
    query::RayCast,
    shape::{Ball, Plane as CollidePlane},
};
use std::rc::Rc;
#[derive(Copy, Clone, Debug, PartialEq)]
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
    node: Node,
    collision_constraint: CollisionConstraint,
    transform: Isometry3<f32>,
    offset: Vector3<f32>,
}

impl Gizmo {
    pub fn new(node: Node) -> Self {
        Self {
            node,
            collision_constraint: CollisionConstraint::None,
            transform: Isometry3::identity(),
            offset: Vector3::identity(),
        }
    }
    pub fn node(&self) -> &Node {
        &self.node
    }
    pub fn rescale(&self, delta: f32) {
        self.node.set_scale(delta / 30.);
    }
    pub fn collision_constraint(&self) -> CollisionConstraint {
        self.collision_constraint
    }
    pub fn copy_location(&self, node: &Node) {
        let v = node.global_position();
        self.node().set_position(v[0], v[1], v[2]);
    }
    pub fn handle_mousedown(&mut self, ray: &Ray<f32>, view: &Viewport) -> bool {
        let target = Ball::new(self.node.scale().x);
        // if the central white ball is clicked
        let gizmo_node_t = self.node.transform().isometry;
        if target.intersects_ray(&gizmo_node_t, ray) {
            self.transform =
                Isometry3::from_parts(gizmo_node_t.translation, view.transform().rotation);
            self.collision_constraint = CollisionConstraint::ViewPlane;
            self.node().change_color([1., 1., 1.]);
            return true;
        }
        // if the arrows are clicked
        for child in self.node.owned_children() {
            let g_c = child.owned_children();
            if g_c.len() > 0 {
                let (tip, stem) = (&g_c[1], &g_c[0]);
                let mut collided = false;
                if let Some(_) = tip.collides_w_ray(&ray) {
                    collided = true
                } else if let Some(_) = stem.collides_w_ray(&ray) {
                    collided = true
                }
                let (color, constraint) = match child.info().name.as_str() {
                    "x-axis" => ([1., 0., 0.], CollisionConstraint::XAxis),
                    "y-axis" => ([0., 1., 0.], CollisionConstraint::YAxis),
                    "z-axis" => ([0., 0., 1.], CollisionConstraint::ZAxis),
                    _ => ([0., 0., 0.], CollisionConstraint::None),
                };
                if collided {
                log!("collided with arrow");
                    self.transform = Isometry3::identity();
                    self.collision_constraint = constraint;
                    stem.change_color(color);
                    tip.change_color(color);
                    return true;
                }
            } else {
                // if the cuboids are clicked
                if let Some(transform) = child.collides_w_ray(&ray) {
                    log!("collided with cuboid");
                    let (color, constraint) = match child.info().name.as_str() {
                        "pan_x" => ([1., 0., 0.], CollisionConstraint::XPlane),
                        "pan_y" => ([0., 1., 0.], CollisionConstraint::YPlane),
                        "pan_z" => ([0., 0., 1.], CollisionConstraint::ZPlane),
                        _ => ([0., 0., 0.], CollisionConstraint::None),
                    };
                    self.collision_constraint = constraint;
                    self.transform = transform;
                    self.offset =
                        transform.translation.vector - Vector3::from(self.node.global_position());
                    child.change_color(color);
                    return true;
                }
            }
        }
        return false;
    }
    pub fn handle_mousemove(&self, ray: &Ray<f32>, active_node: &Option<Rc<Node>>) {
        let collider = CollidePlane::new(match self.collision_constraint {
            CollisionConstraint::YAxis | CollisionConstraint::XPlane => Vector3::x_axis(),
            CollisionConstraint::XAxis
            | CollisionConstraint::ZAxis
            | CollisionConstraint::YPlane => Vector3::y_axis(),
            _ => Vector3::z_axis(),
        });
        if let Some(i) = collider.toi_and_normal_with_ray(&self.transform, &ray, false) {
            if let Some(node) = active_node.as_ref() {
                let pos = node.position();
                // do calculation relative to parent element
                let poi = ray.point_at(i.toi);
                let p_t = node.parent_transform();
                let p_v = p_t.isometry.translation.vector;
                let p_r = p_t.isometry.rotation;
                let p_diff = Vector3::new(poi.x, poi.y, poi.z) - p_v;
                let p = divide(p_diff, p_t.scale);
                let poi = p_r.inverse().transform_vector(&p);
                let p = match self.collision_constraint {
                    CollisionConstraint::XAxis => [poi.x, pos[1], pos[2]],
                    CollisionConstraint::YAxis => [pos[0], poi.y, pos[2]],
                    CollisionConstraint::ZAxis => [pos[0], pos[1], poi.z],
                    _ => [poi.x, poi.y, poi.z],
                };
                //let o = multiply(p_t.scale, offset.clone());
                let o = self.offset;
                self.copy_location(&node);
                node.set_position(p[0] - o.x, p[1] - o.y, p[2] - o.z);
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
        let mut nodes = vec![&self.node];
        if self.collision_constraint != CollisionConstraint::ViewPlane {
            for child in self.node.owned_children() {
                let g_c = child.owned_children();
                let name = child.info().name;
                if g_c.len() > 0 {
                    let n_n = vec![&g_c[0], &g_c[1]];
                    if self.collision_constraint == CollisionConstraint::XAxis && name == "x-axis" {
                        nodes = n_n;
                        break;
                    } else if self.collision_constraint == CollisionConstraint::YAxis
                        && name == "y-axis"
                    {
                        nodes = n_n;
                        break;
                    } else if self.collision_constraint == CollisionConstraint::ZAxis
                        && name == "z-axis"
                    {
                        nodes = n_n;
                        break;
                    }
                } else {
                    if self.collision_constraint == CollisionConstraint::XPlane && name == "pan_x" {
                        nodes = vec![child];
                        break;
                    } else if self.collision_constraint == CollisionConstraint::YPlane
                        && name == "pan_y"
                    {
                        nodes = vec![child];
                        break;
                    } else if self.collision_constraint == CollisionConstraint::ZPlane
                        && name == "pan_z"
                    {
                        nodes = vec![child];
                        break;
                    }
                }
            }
        }
        self.collision_constraint = CollisionConstraint::None;
        self.transform = Isometry3::identity();
        for each in nodes {
            each.change_color(color);
        }
    }
}
