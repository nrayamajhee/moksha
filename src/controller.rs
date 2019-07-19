use crate::{mesh::Transform, renderer::Renderer};
use nalgebra::{
    Isometry3, Matrix4, Orthographic3, Perspective3, Point3, Unit, UnitQuaternion, Vector3,
};
use std::f32::consts::PI;

#[derive(Copy, Clone)]
pub enum MouseButton {
    LEFT = 0,
    MIDDLE = 1,
    RIGHT = 2,
}

#[derive(Debug, Copy, Clone)]
pub struct ProjectionConfig {
    fov: f32,
    near: f32,
    far: f32,
}

#[derive(Debug, Copy, Clone)]
pub enum Projection {
    Orthographic(Orthographic3<f32>),
    Perspective(Perspective3<f32>),
}

pub struct Viewport {
    proj_config: ProjectionConfig,
    initial_config: ProjectionConfig,
    proj: Projection,
    view: Isometry3<f32>,
    initial_view: Isometry3<f32>,
    target: Point3<f32>,
    aspect_ratio: f32,
    speed: f32,
    button: Option<MouseButton>,
    rotate: bool,
}

impl Viewport {
    pub fn new(aspect_ratio: f32) -> Self {
        let fov = PI / 3.;
        let (near, far) = (0.1, 100.);
        let persp = Perspective3::new(aspect_ratio, fov, near, far);
        let proj = Projection::Perspective(persp);
        let proj_config = ProjectionConfig { fov, near, far };

        let pos = Point3::new(0.0, 0.0, 10.0);
        let target = Point3::new(0.0, 0.0, 0.0);
        let up = Vector3::y();
        let view = Isometry3::look_at_rh(&pos, &target, &up);

        let button = Some(MouseButton::LEFT);
        let rotate = false;

        Self {
            initial_config: proj_config.clone(),
            proj_config,
            proj,
            initial_view: view.clone(),
            view,
            aspect_ratio,
            target,
            speed: 1.0,
            button,
            rotate,
        }
    }
    pub fn view(&self) -> Matrix4<f32> {
        self.view.to_homogeneous()
    }
    pub fn proj(&self) -> Matrix4<f32> {
        match self.proj {
            Projection::Orthographic(proj) => Matrix4::from(proj),
            Projection::Perspective(proj) => Matrix4::from(proj),
        }
    }
    pub fn update_rot(&mut self, dx: i32, dy: i32, dt: f32) {
        if self.rotate {
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
    }
    pub fn update_zoom(&mut self, ds: f32) {
        let d = 0.1;
        let delta = if ds < 0. { 1. - d } else { 1. + d };
        self.view.translation.vector = self.speed * delta * self.view.translation.vector;
        if let Projection::Orthographic(_) = self.proj {
            self.update_proj(false)
        }
    }
    pub fn reset(&mut self) {
        self.view = self.initial_view;
        if let Projection::Orthographic(_) = self.proj {
            self.update_proj(false);
        }
    }
    pub fn resize(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
        self.update_proj(false);
    }
    pub fn update_proj(&mut self, perspective: bool) {
        self.proj = if perspective {
            Projection::Perspective(Perspective3::new(
                self.aspect_ratio,
                self.proj_config.fov,
                self.proj_config.near,
                self.proj_config.far,
            ))
        } else {
            let halfy = self.proj_config.fov / 2.;
            let height = self.view.translation.vector.magnitude() * halfy.tan();
            let width = height * self.aspect_ratio;
            Projection::Orthographic(Orthographic3::new(
                -width,
                width,
                -height,
                height,
                -self.proj_config.far,
                self.proj_config.far,
            ))
        };
    }
    pub fn switch_projection(&mut self) {
        match self.proj {
            Projection::Perspective(_) => {
                self.update_proj(false);
            }
            Projection::Orthographic(_) => {
                self.update_proj(true);
            }
        }
    }
    pub fn button(&self) -> Option<MouseButton> {
        self.button
    }
    pub fn disable_rotation(&mut self) {
        self.rotate = false;
    }
    pub fn enable_rotation(&mut self) {
        self.rotate = true;
    }
}
