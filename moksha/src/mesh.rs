use crate::{log, renderer::ShaderType, Color};
use genmesh::{
    generators::{IndexedPolygon, SharedVertex},
    EmitTriangles, Triangulate, Vertex,
};
use nalgebra::{one, Isometry3, Matrix4, Point3, Translation3, Vector3};
use std::collections::HashMap;
use wavefront_obj::{mtl, obj};

/// A 3D mesh containing geometry and material.
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    pub geometry: Geometry,
    pub material: Material,
}

/// Geometry of a 3D object containing vertices, indices, and face normals.
#[derive(Debug, Clone, PartialEq)]
pub struct Geometry {
    pub vertices: Vec<f32>,
    pub indices: Vec<u16>,
    pub normals: Vec<f32>,
}

/// Material for a 3D object; can contain either color, vertex colors, or texture.
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub shader_type: ShaderType,
    pub flat_shade: bool,
    pub wire_overlay: Option<Color>,
    pub outline: Option<f32>,
    pub color: Option<Color>,
    pub vertex_colors: Option<Vec<f32>>,
    pub tex_type: TextureType,
    pub tex_coords: Option<Vec<f32>>,
    pub texture_urls: Vec<String>,
    pub texture_indices: Vec<usize>,
}

/// A 3D transform that can handle translation, rotation, and non-uniform scaling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub isometry: Isometry3<f32>,
    pub scale: Vector3<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextureType {
    Tex2d,
    CubeMap,
    None,
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
    pub fn from_obj(obj: &obj::Object) -> Self {
        let mut buf_vertices = Vec::new();
        for vertex in &obj.vertices {
            buf_vertices.push(vertex.x);
            buf_vertices.push(vertex.y);
            buf_vertices.push(vertex.z);
        }
        let mut buf_normals = Vec::new();
        for normal in &obj.normals {
            buf_normals.push(normal.x);
            buf_normals.push(normal.y);
            buf_normals.push(normal.z);
        }
        let mut buf_tex_coords = Vec::new();
        for normal in &obj.normals {
            buf_tex_coords.push(normal.x);
            buf_tex_coords.push(normal.y);
            buf_tex_coords.push(normal.z);
        }
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();
        for shape in &obj.geometry[0].shapes {
            if let obj::Primitive::Triangle(a, b, c) = shape.primitive {
                if a.2 != None {
                    for (a, na) in [a.0, b.0, c.0]
                        .iter()
                        .zip([a.2.unwrap(), b.2.unwrap(), c.2.unwrap()].iter())
                    {
                        let (e, ne) = (*a * 3, *na);
                        indices.push(e as u16);
                        vertices.push(buf_vertices[e] as f32);
                        vertices.push(buf_vertices[e + 1] as f32);
                        vertices.push(buf_vertices[e + 2] as f32);
                        normals.push(buf_vertices[e] as f32);
                        normals.push(buf_vertices[e + 1] as f32);
                        normals.push(buf_vertices[e + 2] as f32);
                    }
                } else {
                    log!("obj file doesn't have normals. Only vertices are loaded");
                }
            }
        }
        Self {
            vertices,
            indices,
            normals,
        }
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

impl Default for Material {
    fn default() -> Self {
        Self {
            shader_type: ShaderType::Simple,
            flat_shade: false,
            wire_overlay: None,
            outline: None,
            color: None,
            vertex_colors: None,
            tex_type: TextureType::None,
            tex_coords: None,
            texture_urls: Vec::new(),
            texture_indices: Vec::new(),
        }
    }
}

impl Material {
    pub fn new_color_no_shade(color: Color) -> Self {
        Self::default().color(color)
    }
    pub fn new_color(color: Color) -> Self {
        Self::default().color(color).shader_type(ShaderType::Color)
    }
    pub fn new_wire(color: Color) -> Self {
        Self::default()
            .color(color)
            .wire_overlay_colored(color)
            .shader_type(ShaderType::Wireframe)
    }
    pub fn new_texture(url: &str, tex_coords: Vec<f32>) -> Self {
        Self::new_color(Color::rgb(1., 1., 1.))
            .tex_type(TextureType::Tex2d)
            .tex_coords(tex_coords)
            .texture(url)
    }
    pub fn new_cube_map(urls: [&str; 6]) -> Self {
        let mut mat = Self::new_color(Color::rgb(1., 1., 1.))
            .shader_type(ShaderType::CubeMap)
            .tex_type(TextureType::CubeMap);
        for each in urls.iter() {
            mat = mat.texture(each);
        }
        mat
    }
    pub fn from_obj(
        dir: &str,
        obj: &obj::Object,
        mat_set: &Option<mtl::MtlSet>,
        img_obj_url: Option<&HashMap<String, String>>,
    ) -> Self {
        let mut material = Self::new_color(Color::white());
        if obj.geometry[0].shapes[0].smoothing_groups.is_empty() {
            log!("No smooting group found. Mesh will be rendered flat.");
            material = material.flat();
        }
        let mut tex_coords = Vec::new();
        for shape in &obj.geometry[0].shapes {
            if let obj::Primitive::Triangle(a, b, c) = shape.primitive {
                if a.1 != None {
                    for each in [a.2.unwrap(), b.2.unwrap(), c.2.unwrap()].iter() {
                        let u = obj.tex_vertices[*each].u;
                        let v = -obj.tex_vertices[*each].v;
                        tex_coords.push(u as f32);
                        tex_coords.push(v as f32);
                    }
                } else {
                    log!("obj file doesn't have texture coordinates!");
                }
            }
        }
        if let Some(material_name) = &obj.geometry[0].material_name {
            if let Some(mat_set) = mat_set {
                for each in &mat_set.materials {
                    if &each.name == material_name {
                        let c = each.color_diffuse;
                        material = Self::new_color(Color::rgba(
                            c.r as f32,
                            c.g as f32,
                            c.b as f32,
                            each.alpha as f32,
                        ));
                        if let Some(name) = &each.uv_map {
                            let url = if let Some(urls) = img_obj_url {
                                if let Some(url) = urls.get(name) {
                                    Some(url.to_string())
                                } else {
                                    log!("The file with name " name.to_string() " was not uploaded. Resorting to color");
                                    None
                                }
                            } else {
                                if dir != "" {
                                    Some(format!("{}/{}", dir, name))
                                } else {
                                    log!("Invalid texture path! Won't load any texture for " name.to_string() ".");
                                    None
                                }
                            };
                            if let Some(url) = url {
                                material = material
                                    .tex_type(TextureType::Tex2d)
                                    .tex_coords(tex_coords)
                                    .texture(&url);
                            }
                        }
                        return material;
                    }
                }
            }
        }
        material
    }
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
    pub fn flat(mut self) -> Self {
        self.flat_shade = true;
        self
    }
    pub fn outline(mut self) -> Self {
        self.outline = Some(5.);
        self
    }
    pub fn shader_type(mut self, shader: ShaderType) -> Self {
        self.shader_type = shader;
        self
    }
    pub fn tex_type(mut self, tex_type: TextureType) -> Self {
        self.tex_type = tex_type;
        self
    }
    pub fn wire_overlay(mut self) -> Self {
        let color = if let Some(c) = self.color {
            Color::from(c)
        } else {
            Color::rgb(1., 1., 1.)
        };
        self.wire_overlay_colored(color)
    }
    pub fn wire_overlay_colored(mut self, color: Color) -> Self {
        self.wire_overlay = Some(color);
        self
    }
    pub fn tex_coords(mut self, tex_coords: Vec<f32>) -> Self {
        self.tex_coords = Some(tex_coords);
        self
    }
    pub fn texture(mut self, url: &str) -> Self {
        self.texture_urls.push(String::from(url));
        self
    }
    pub fn vertex_colors(vertex_color: Vec<f32>) -> Self {
        let mut mat = Self::default();
        mat.vertex_colors = Some(vertex_color);
        mat
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
    pub fn from_scale(scale: f32) -> Self {
        Self {
            isometry: Isometry3::identity(),
            scale: Vector3::new(scale, scale, scale),
        }
    }
    pub fn from_scale_vec(x: f32, y: f32, z: f32) -> Self {
        Self {
            isometry: Isometry3::identity(),
            scale: Vector3::new(x, y, z),
        }
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
