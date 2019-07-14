use crate::renderer::{bind_texture, DrawMode, ShaderType};
use genmesh::{
    generators::{IndexedPolygon, SharedVertex},
    EmitTriangles, Triangulate, Vertex,
};
use nalgebra::{one, Isometry3, Vector3, Matrix4};
use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext as GL;

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub isometry: Isometry3<f32>,
    pub scale: Vector3<f32>,
}

impl Transform {
    pub fn to_homogeneous(&self) -> Matrix4<f32> {
        self.isometry.to_homogeneous() * Matrix4::new_nonuniform_scaling(&self.scale) 
    }
}

impl std::ops::Mul<Transform> for Transform {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        let isometry = self.isometry * rhs.isometry;
        // let scale = rhs.scale.cross(&self.scale);
        let scale = rhs.scale;
        Self {
            isometry,
            scale,
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            isometry: one(),
            scale: Vector3::new(1.0,1.0,1.0),
        }
    }
}


#[derive(Debug, Clone)]
pub struct Mesh {
    pub geometry: Geometry,
    pub material: Material,
}

#[derive(Debug, Clone)]
pub struct Geometry {
    pub vertices: Vec<f32>,
    pub indices: Vec<u16>,
    pub normals: Vec<f32>,
}

impl Geometry {
    pub fn from_genmesh<T, P>(primitive: &T) -> Self
    where
        P: EmitTriangles<Vertex = usize>,
        T: SharedVertex<Vertex> + IndexedPolygon<P>,
    {
        Self::generate(primitive, true)
    }
    pub fn from_genmesh_no_normals<T, P>(primitive: &T) -> Self
    where
        P: EmitTriangles<Vertex = usize>,
        T: SharedVertex<Vertex> + IndexedPolygon<P>,
    {
        Self::generate(primitive, false)
    }
    pub fn generate<T, P>(primitive: &T, add_normals: bool) -> Self
    where
        P: EmitTriangles<Vertex = usize>,
        T: SharedVertex<Vertex> + IndexedPolygon<P>,
    {
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();
        for Vertex { pos, normal } in primitive.shared_vertex_iter() {
            vertices.push(pos.x);
            vertices.push(pos.y);
            vertices.push(pos.z);
            if add_normals {
                normals.push(normal.x);
                normals.push(normal.y);
                normals.push(normal.z);
            }
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

#[derive(Debug, Clone)]
pub struct Material {
    pub shader_type: ShaderType,
    pub color: Option<[f32; 4]>,
    pub vertex_colors: Option<Vec<f32>>,
    pub tex_coords: Option<Vec<f32>>,
}

impl Material {
    pub fn from_image_texture(gl: &GL, url: &str, tex_coords: Vec<f32>) -> Result<Self, JsValue> {
        bind_texture(gl, url)?;
        let tex_coords = Some(tex_coords);
        Ok(Self {
            shader_type: ShaderType::Texture,
            color: None,
            vertex_colors: None,
            tex_coords,
        })
    }
    pub fn single_color(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            shader_type: ShaderType::Color,
            color: Some([r,g,b,a]),
            vertex_colors: None,
            tex_coords: None,
        }
    }
    pub fn single_color_no_shade(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            shader_type: ShaderType::Simple,
            color: Some([r,g,b,a]),
            vertex_colors: None,
            tex_coords: None,
        }
    }
    pub fn vertex_colors(vertex_color: Vec<f32>) -> Self {
        Self {
            shader_type: ShaderType::VertexColor,
            color: None,
            vertex_colors: Some(vertex_color),
            tex_coords: None,
        }
    }
}
