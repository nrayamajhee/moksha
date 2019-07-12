use crate::renderer::{bind_texture, DrawMode, ShaderType};
use genmesh::{
    generators::{IndexedPolygon, SharedVertex},
    EmitTriangles, Triangulate, Vertex,
};
use nalgebra::{Similarity3, UnitQuaternion, Vector3, one, zero};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext as GL;

pub type Transform = Similarity3<f32>;

pub struct Node {
    index: usize,
    storage: Rc<RefCell<Storage>>,
    children: Option<Vec<Node>>,
}

impl Node {
    pub fn set_position(&self, pos: [f32; 3]) {
        let mut storage = self.storage.borrow_mut();
        let transform = storage.get_mut_transform(self.index);
        transform.isometry.translation.vector = Vector3::new(pos[0], -pos[1], pos[2]);
    }
    pub fn rotate(&self, rot: UnitQuaternion<f32>) {
        let mut storage = self.storage.borrow_mut();
        let transform = storage.get_mut_transform(self.index);
        transform.append_rotation_mut(&rot);
    }
    pub fn get_position(&self) -> Vector3<f32> {
        let mut storage = self.storage.borrow_mut();
        let transform = storage.get_mut_transform(self.index);
        transform.isometry.translation.vector
    }
    pub fn rotation(&self) -> UnitQuaternion<f32> {
        let mut storage = self.storage.borrow_mut();
        let transform = storage.get_mut_transform(self.index);
        transform.isometry.rotation
    }
    pub fn set_scale(&self, scale: f32) {
        let mut storage = self.storage.borrow_mut();
        let transform = storage.get_mut_transform(self.index);
        transform.set_scaling(1. / scale);
    }
    pub fn scale(&self) -> f32 {
        let mut storage = self.storage.borrow_mut();
        let transform = storage.get_mut_transform(self.index);
        transform.scaling()
    }
}

pub struct Scene {
    root: Node,
}

impl Scene {
    pub fn new() -> Self {
        let storage = Rc::new(RefCell::new(Storage::new()));
        Self {
            root: Self::object_default(storage.clone()),
        }
    }
    pub fn get_storage(&self) -> Rc<RefCell<Storage>> {
        self.root.storage.clone()
    }
    pub fn add(&self, node: &Node) {
        self.add_with_mode(node, DrawMode::Triangle);
    }
    pub fn add_with_mode(&self, node: &Node, mode: DrawMode) {
        let storage = self.get_storage();
        let mut storage = storage.borrow_mut();
        let info = storage.get_mut_info(node.index);
        info.draw_mode = mode;
    }
    fn object_default(storage: Rc<RefCell<Storage>>) -> Node {
        Self::object(storage, None, one())
    }
    pub fn object(storage: Rc<RefCell<Storage>>, mesh: Option<Mesh>, transform: Transform) -> Node {
        let sto = storage.clone();
        let mut a_storage = sto.borrow_mut();
        let index = a_storage.add(mesh, transform, "node");
        Node {
            index,
            storage,
            children: None,
        }
    }
    pub fn empty(&self) -> Node {
        Self::object_default(self.get_storage())
    }
    pub fn object_from_mesh(&self, geometry: Geometry, material: Material) -> Node {
        Self::object(
            self.get_storage(),
            Some(Mesh { geometry, material }),
            one(),
        )
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub geometry: Geometry,
    pub material: Material,
}

#[derive(Debug, Clone)]
pub struct ObjectInfo {
    pub name: String,
    pub draw_mode: DrawMode,
}

#[derive(Debug)]
pub struct Storage {
    info: Vec<ObjectInfo>,
    meshes: Vec<Option<Mesh>>,
    transforms: Vec<Transform>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            info: Vec::new(),
            meshes: Vec::new(),
            transforms: Vec::new(),
        }
    }
    pub fn add(&mut self, mesh: Option<Mesh>, transform: Transform, name: &str) -> usize {
        let index = self.meshes.len();
        self.meshes.push(mesh);
        self.transforms.push(transform);
        self.info.push(ObjectInfo {
            name: String::from(name),
            draw_mode: DrawMode::None,
        });
        index
    }
    pub fn get_mut_transform(&mut self, indx: usize) -> &mut Transform {
        self.transforms
            .get_mut(indx)
            .expect("No such transform found!")
    }
    pub fn get_transform(&self, indx: usize) -> Transform {
        *self.transforms.get(indx).expect("No such transform found!")
    }
    pub fn meshes(&self) -> &Vec<Option<Mesh>> {
        &self.meshes
    }
    pub fn get_info(&self, indx: usize) -> ObjectInfo {
        self.info.get(indx).expect("No node info found!").clone()
    }
    pub fn get_mut_info(&mut self, indx: usize) -> &mut ObjectInfo {
        self.info.get_mut(indx).expect("No node info found!")
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
    pub fn single_color_no_shade(color: [f32; 4]) -> Self {
        Self {
            shader_type: ShaderType::Simple,
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
