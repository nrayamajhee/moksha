#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DrawMode {
    Points,
    Lines,
    Triangle,
    Arrays,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RenderFlags {
    pub blend: bool,
    pub depth: bool,
    pub stencil: bool,
    pub cull_face: bool,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RenderConfig {
    pub depth_fn: u32,
    pub front_face: u32,
    pub cull_face: u32,
}

