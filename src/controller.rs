use crate::{renderer::Renderer,mesh::Transform};
use nalgebra::{Isometry3, Matrix4, Point3, Perspective3, Vector3, Unit, UnitQuaternion};
use std::f32::consts::PI;

pub enum MouseButton {
    LEFT,
    MIDDLE,
    RIGHT
}

pub struct Viewport {
    proj: Perspective3<f32>,
    fov: f32,
    view: Isometry3<f32>,
    initial_view: Isometry3<f32>,
    target: Point3<f32>,
    speed: f32,
    button: Option<MouseButton>,
}

impl Viewport {
    pub fn new(renderer: &Renderer) -> Self {
        let fov = PI / 3.;
        let proj = Perspective3::new(renderer.aspect_ratio(), fov, 0.01, 100.);
        let pos = Point3::new(20.0, 20.0, -20.0);
        let target = Point3::new(0.0, 0.0, 0.0);
        let up = Vector3::y();
        let view = Isometry3::look_at_rh(&pos, &target, &up);
        let button = Some(MouseButton::LEFT);
        Self {
            proj,
            initial_view: view.clone(),
            view,
            fov,
            target,
            speed: 1.0,
            button,
        }
    }
    pub fn view(&self) -> Matrix4<f32> {
        self.view.to_homogeneous()
    }
    pub fn proj(&self) -> Matrix4<f32> {
        Matrix4::from(self.proj)
    }
    pub fn update_rot(&mut self, dx: i32, dy: i32, dt: f32) {
        let pitch = dy as f32 * 0.01 * self.speed;
        let yaw = dx as f32 * 0.01 * self.speed;
        let delta_rot = {
            let axis = Unit::new_normalize(self.view.rotation.conjugate() * Vector3::x());
            let q_ver = UnitQuaternion::from_axis_angle(&axis, pitch);
            let axis = Unit::new_normalize(Vector3::y());
            let q_hor = UnitQuaternion::from_axis_angle(&axis, yaw);
            q_ver * q_hor
        };
        self.view.rotation *= &delta_rot;
    }
    pub fn update_zoom(&mut self, ds: f32) {
        let d = 0.1;
        let delta = if ds < 0. {
            1. - d
        } else {
            1. + d
        };
        self.view.translation.vector = self.speed * delta * self.view.translation.vector;
    }
    pub fn reset(&mut self) {
        self.view = self.initial_view;
    }
    pub fn update_proj(&mut self, aspect_ratio: f32) {
        self.proj = Perspective3::new(aspect_ratio, self.fov, 0.1, 100.);
    }
}
