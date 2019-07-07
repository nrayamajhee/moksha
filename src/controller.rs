use cgmath::Matrix4;

pub struct Viewport {
    pub proj: Matrix4<f32>,
    pub view: Matrix4<f32>,
}
