use cgmath::Matrix4;
use web_sys::{WebGlProgram, WebGlRenderingContext as GL, WebGlTexture};
use wasm_bindgen::JsValue;

mod renderer;
mod shader;

pub use renderer::{Config, Renderer};
pub use shader::{bind_matrix, ShaderType, bind_texture};

use genmesh::{
    generators::{IndexedPolygon, SharedVertex},
    EmitTriangles, Triangulate, Vertex,
};

pub struct Geometry {
    pub vertices: Vec<f32>,
    pub indices: Vec<u16>,
    pub normals: Vec<f32>,
}

pub struct Viewport {
    pub proj: Matrix4<f32>,
    pub view: Matrix4<f32>,
}

impl Geometry {
    pub fn from_genmesh<T, P>(primitive: &T) -> Self
    where
        P: EmitTriangles<Vertex = usize>,
        T: SharedVertex<Vertex> + IndexedPolygon<P>,
    {
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();
        for Vertex { pos, normal } in primitive.shared_vertex_iter() {
            vertices.push(pos.x * 1.0);
            vertices.push(pos.y * 1.0);
            vertices.push(pos.z * 1.0);
            normals.push(normal.x);
            normals.push(normal.y);
            normals.push(normal.z);
        }
        for t in primitive.indexed_polygon_iter().triangulate() {
            indices.push(t.x as u16);
            indices.push(t.y as u16);
            indices.push(t.z as u16);
        }
        Self {
            vertices,
            indices,
            normals,
        }
    }
}


pub struct Material {
    shader_type: ShaderType,
    color: Option<[f32; 4]>,
    vertex_colors: Option<Vec<f32>>,
    tex_coords: Option<Vec<f32>>,
}

impl Material {
    pub fn from_image_texture(gl: &GL, url: &str, tex_coords: Vec<f32>) -> Result<Self, JsValue> {
        let texture = bind_texture(gl, url)?;
        let tex_coords = Some(tex_coords);
        Ok(Self {
            shader_type: ShaderType::Texture,
            color: None,
            vertex_colors: None,
            tex_coords
        })
    }
    pub fn color(gl: &GL, color: [f32;4]) -> Self {
        Self {
            shader_type: ShaderType::Color,
            color: Some(color),
            vertex_colors: None,
            tex_coords: None,
        }
    }
}