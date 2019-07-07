use crate::renderer::{bind_texture, ShaderType};
use cgmath::{Decomposed, One, Quaternion, Vector3, Zero};
use genmesh::{
    generators::{IndexedPolygon, SharedVertex},
    EmitTriangles, Triangulate, Vertex,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use strum::IntoEnumIterator;
use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext as GL;

type Transform = Decomposed<Vector3<f32>, Quaternion<f32>>;

pub struct Object {
    shader_type: ShaderType,
    index: usize,
    storage: Rc<RefCell<Storage>>,
}

impl Object {
    pub fn new_with_transform(
        storage: Rc<RefCell<Storage>>,
        mesh: Mesh,
        transform: Transform,
    ) -> Self {
        let storage = storage.clone();
        let mut mesh_storage = storage.borrow_mut();
        let (shader_type, index) = mesh_storage.add(mesh, transform);
        Self {
            shader_type,
            index,
            storage: storage.clone(),
        }
    }
    pub fn new(storage: Rc<RefCell<Storage>>, geometry: Geometry, material: Material) -> Self {
        let transform: Decomposed<Vector3<f32>, Quaternion<f32>> = Decomposed {
            disp: Vector3::zero(),
            rot: Quaternion::one(),
            scale: 1.0,
        };
        Self::new_with_transform(storage, Mesh { geometry, material }, transform)
    }
    pub fn set_position(&self, pos: [f32; 3]) {
        let mut storage = self.storage.borrow_mut();
        let transform = storage.get_mut_transform(self.shader_type, self.index);
        transform.disp = Vector3::from(pos);
    }
    pub fn set_rotation(&self, rot: Quaternion<f32>) {
        let mut storage = self.storage.borrow_mut();
        let transform = storage.get_mut_transform(self.shader_type, self.index);
        transform.rot = rot;
    }
    pub fn set_scale(&self, scale: f32) {
        let mut storage = self.storage.borrow_mut();
        let transform = storage.get_mut_transform(self.shader_type, self.index);
        transform.scale = scale;
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub geometry: Geometry,
    pub material: Material,
}

#[derive(Debug)]
pub struct Storage {
    meshes: HashMap<ShaderType, Vec<Mesh>>,
    transforms: HashMap<ShaderType, Vec<Transform>>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            meshes: HashMap::new(),
            transforms: HashMap::new(),
        }
    }
    pub fn add(&mut self, mesh: Mesh, transform: Transform) -> (ShaderType, usize) {
        let mut index = 0;
        let shader_type = mesh.material.shader_type;
        if let Some(meshes) = self.meshes.get_mut(&shader_type) {
            index = meshes.len();
            meshes.push(mesh);
        } else {
            self.meshes.insert(shader_type, vec![mesh]);
        }
        if let Some(transforms) = self.transforms.get_mut(&shader_type) {
            transforms.push(transform);
        } else {
            self.transforms.insert(shader_type, vec![transform]);
        }
        (shader_type, index)
    }
    pub fn get_mut_transform(&mut self, shader_type: ShaderType, indx: usize) -> &mut Transform {
        let target = self
            .transforms
            .get_mut(&shader_type)
            .expect("There is no transform storage for this mesh!");
        let target = target
            .get_mut(indx)
            .expect(format!("No transform found at index: {}", indx).as_str());
        target
    }
    pub fn get_transform(&self, shader_type: ShaderType, indx: usize) -> Transform {
        let target = self
            .transforms
            .get(&shader_type)
            .expect("There is no transform storage for this mesh!");
        let transform = target
            .get(indx)
            .expect(format!("No transform found at index: {}", indx).as_str());
        *transform
    }
    pub fn get_meshes(&self, shader_type: &ShaderType) -> Option<&Vec<Mesh>> {
        self.meshes.get(&shader_type)
    }
    pub fn get_transforms(&self, shader_type: &ShaderType) -> Option<&Vec<Transform>> {
        self.transforms.get(&shader_type)
    }
}

#[derive(Clone, Debug)]
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
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();
        for Vertex { pos, normal } in primitive.shared_vertex_iter() {
            vertices.push(pos.x);
            vertices.push(pos.y);
            vertices.push(pos.z);
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
    pub fn single_color(color: [f32; 4]) -> Self {
        Self {
            shader_type: ShaderType::Color,
            color: Some(color),
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
