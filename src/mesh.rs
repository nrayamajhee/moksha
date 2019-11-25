use crate::renderer::{ShaderType};
use genmesh::{
    generators::{IndexedPolygon, SharedVertex},
    EmitTriangles, Triangulate, Vertex,
};
use nalgebra::{one, Isometry3, Matrix4, Point3, Translation3, Vector3};
use wasm_bindgen::JsValue;

/// A 3D transform that can handle translation, rotation, and non-uniform scaling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub isometry: Isometry3<f32>,
    pub scale: Vector3<f32>,
}

impl Transform {
    pub fn to_homogeneous(&self) -> Matrix4<f32> {
        self.isometry.to_homogeneous() * Matrix4::new_nonuniform_scaling(&self.scale)
    }
    pub fn inverse(&self) -> Self {
        Self {
            isometry: self.isometry.inverse(),
            scale: divide([1., 1., 1.].into(), self.scale),
        }
    }
    pub fn transform_vector(&self, vec: &Vector3<f32>) -> Vector3<f32> {
        multiply(self.scale, self.isometry.transform_vector(vec))
    }
    pub fn transform_point(&self, point: &Point3<f32>) -> Point3<f32> {
        let p = self.isometry.transform_point(point);
        let v = multiply(self.scale, Vector3::new(p.x, p.y, p.z));
        Point3::new(v.x, v.y, v.z)
    }
    pub fn identity() -> Self {
        Self::from(Isometry3::identity())
    }
}

impl From<Isometry3<f32>> for Transform {
    fn from(isometry: Isometry3<f32>) -> Self {
        Self {
            isometry,
            scale: Vector3::new(1., 1., 1.),
        }
    }
}

/// Computes a direct product of two vector3s i.e. (a,b,c) x (a',b',c') => (aa', bb', cc')
pub fn multiply(left: Vector3<f32>, right: Vector3<f32>) -> Vector3<f32> {
    Vector3::new(left.x * right.x, left.y * right.y, left.z * right.z)
}

/// Computes a direct division of two vector3s i.e. (a,b,c) x (a',b',c') => (a/a', b/b', c/c')
pub fn divide(left: Vector3<f32>, right: Vector3<f32>) -> Vector3<f32> {
    Vector3::new(left.x / right.x, left.y / right.y, left.z / right.z)
}

impl std::ops::Mul<Transform> for Transform {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        let scale = multiply(self.scale, rhs.scale);
        let shift = multiply(
            self.scale,
            self.isometry
                .rotation
                .transform_vector(&rhs.isometry.translation.vector),
        );
        let isometry = Isometry3::from_parts(
            #[allow(clippy::suspicious_arithmetic_impl)]
            Translation3::from(self.isometry.translation.vector + shift),
            self.isometry.rotation * rhs.isometry.rotation,
        );
        Self { isometry, scale }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            isometry: one(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

/// A 3D mesh containing geometry and material.
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    pub geometry: Geometry,
    pub material: Material,
}

impl Mesh {
    pub fn new(geometry: Geometry, material: Material) -> Self {
        Self { geometry, material }
    }
    pub fn setup_unique_vertices(&mut self) {
        let mut vertices = Vec::new();
        for each in self.geometry.indices.iter() {
            let i = (each * 3) as usize;
            vertices.push(self.geometry.vertices[i]);
            vertices.push(self.geometry.vertices[i + 1]);
            vertices.push(self.geometry.vertices[i + 2]);
        }
        if !self.geometry.normals.is_empty() {
            let mut normals = Vec::new();
            for each in self.geometry.indices.iter() {
                let i = (each * 3) as usize;
                normals.push(self.geometry.normals[i]);
                normals.push(self.geometry.normals[i + 1]);
                normals.push(self.geometry.normals[i + 2]);
            }
            self.geometry.normals = normals;
        }
        self.geometry.vertices = vertices;
        if let Some(tex_coords) = self.material.tex_coords.as_ref() {
            let mut coords = Vec::new();
            for each in self.geometry.indices.iter() {
                let i = (each * 2) as usize;
                coords.push(tex_coords[i]);
                coords.push(tex_coords[i + 1]);
            }
            self.material.tex_coords = Some(coords);
        }
    }
}

/// Geometry of a 3D object containing vertices, indices, and face normals.
#[derive(Debug, Clone, PartialEq)]
pub struct Geometry {
    pub vertices: Vec<f32>,
    pub indices: Vec<u16>,
    pub normals: Vec<f32>,
}

impl Default for Geometry {
    fn default() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            normals: Vec::new(),
        }
    }
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
    fn generate<T, P>(primitive: &T, add_normals: bool) -> Self
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

/// Material for a 3D object; can contain either color, vertex colors, or texture.
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub shader_type: ShaderType,
    pub flat_shade: bool,
    pub wire_overlay: bool,
    pub color: Option<[f32; 4]>,
    pub vertex_colors: Option<Vec<f32>>,
    pub tex_coords: Option<Vec<f32>>,
    pub texture_urls: Vec<String>,
    pub texture_indices: Vec<usize>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            shader_type: ShaderType::Simple,
            flat_shade: false,
            wire_overlay: false,
            color: None,
            vertex_colors: None,
            tex_coords: None,
            texture_urls: Vec::new(),
            texture_indices: Vec::new(),
        }
    }
}

impl Material {
    pub fn new_color_no_shade(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::default().color(r, g, b, a)
    }
    pub fn new_color(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::default()
            .color(r, g, b, a)
            .shader_type(ShaderType::Color)
    }
    pub fn new_wire(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::new_color(r, g, b, a)
            .wire_overlay()
            .shader_type(ShaderType::Wireframe)
    }
    pub fn new_texture(url: &str, tex_coords: Vec<f32>) -> Self  {
        Self::new_color(1.,1.,1.,1.)
            .texture(url, tex_coords)
    }
    pub fn color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = Some([r, g, b, a]);
        self
    }
    pub fn flat(mut self) -> Self {
        self.flat_shade = true;
        self
    }
    pub fn shader_type(mut self, shader: ShaderType) -> Self {
        self.shader_type = shader;
        self
    }
    pub fn wire_overlay(mut self) -> Self {
        self.wire_overlay = true;
        self
    }
    pub fn texture(mut self, url: &str, tex_coords: Vec<f32>) -> Self {
        self.tex_coords = Some(tex_coords);
        self.texture_urls.push(String::from(url));
        self
    }
    pub fn vertex_colors(vertex_color: Vec<f32>) -> Self {
        let mut mat = Self::default();
        mat.vertex_colors = Some(vertex_color);
        mat
    }
}
