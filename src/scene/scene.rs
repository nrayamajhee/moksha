use crate::{
    mesh::{Geometry, Material, Mesh, Transform},
    renderer::{DrawMode, Renderer},
    RcRcell, rc_rcell,
};
// use genmesh::generators::{Cone, Cylinder, IcoSphere};
use nalgebra::{UnitQuaternion, Vector3};
use std::cell::RefCell;
use web_sys::WebGlVertexArrayObject;
// use std::f32::consts::PI;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectInfo {
    pub name: String,
    pub draw_mode: DrawMode,
}

/// A node is a entity in a scene that holds reference to its props in Storage, keeps tracks of
/// other nodes that are its children either borrowed or owned.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    index: usize,
    storage: RcRcell<Storage>,
    children: Vec<RcRcell<Node>>,
    owned_children: Vec<Node>,
}

impl Node {
    pub fn position(&self) -> Vector3<f32> {
        let mut storage = self.storage.borrow_mut();
        let transform = storage.mut_transform(self.index);
        transform.isometry.translation.vector
    }
    pub fn set_position(&self, pos: [f32; 3]) {
        let p_transform = {
            let mut storage = self.storage.borrow_mut();
            let transform = storage.mut_transform(self.index);
            transform.isometry.translation.vector = Vector3::from(pos);
            *transform
        };
        for child in self.children.iter() {
            child.borrow().apply_parent_transform(p_transform);
        }
        for child in self.owned_children.iter() {
            child.apply_parent_transform(p_transform);
        }
    }
    pub fn set_scale(&self, scale: f32) {
        let p_transform = {
            let mut storage = self.storage.borrow_mut();
            let transform = storage.mut_transform(self.index);
            transform.scale = Vector3::new(scale, scale, scale);
            *transform
        };
        for child in self.children.iter() {
            child.borrow().apply_parent_transform(p_transform);
        }
        for child in self.owned_children.iter() {
            child.apply_parent_transform(p_transform);
        }
    }
    fn apply_parent_transform(&self, transform: Transform) {
        self.set_parent_transform(transform);
        let p_transform = transform * self.transform();
        // log!("Transform\n{:?}\nChild:\nTransform\n{:?}", transform, p_transform);
        for child in self.children.iter() {
            child.borrow().apply_parent_transform(p_transform);
        }
        for child in self.owned_children.iter() {
            child.apply_parent_transform(p_transform);
        }
    }
    pub fn set_rotation(&self, rot: UnitQuaternion<f32>) {
        let p_transform = {
            let mut storage = self.storage.borrow_mut();
            let mut transform = storage.mut_transform(self.index);
            transform.isometry.rotation = rot;
            *transform
        };
        for child in self.children.iter() {
            child.borrow().apply_parent_transform(p_transform);
        }
        for child in self.owned_children.iter() {
            child.apply_parent_transform(p_transform);
        }
    }
    pub fn rotate_by(&self, rot: UnitQuaternion<f32>) {
        let p_transform = {
            let mut storage = self.storage.borrow_mut();
            let transform = storage.mut_transform(self.index);
            transform.isometry.append_rotation_wrt_center_mut(&rot);
            *transform
        };
        for child in self.children.iter() {
            child.borrow().apply_parent_transform(p_transform);
        }
        for child in self.owned_children.iter() {
            child.apply_parent_transform(p_transform);
        }
    }
    pub fn set_transform(&self, transform: Transform) {
        let p_transform = {
            let mut storage = self.storage.borrow_mut();
            let t = storage.mut_transform(self.index);
            *t = transform;
            *t
        };
        for child in self.children.iter() {
            child.borrow().apply_parent_transform(p_transform);
        }
        for child in self.owned_children.iter() {
            child.apply_parent_transform(p_transform);
        }
    }
    pub fn rotation(&self) -> UnitQuaternion<f32> {
        let mut storage = self.storage.borrow_mut();
        let transform = storage.mut_transform(self.index);
        transform.isometry.rotation
    }
    pub fn set_scale_vec(&self, scale: [f32; 3]) {
        let mut storage = self.storage.borrow_mut();
        let transform = storage.mut_transform(self.index);
        transform.scale = Vector3::new(scale[0], scale[2], scale[1]);
    }
    pub fn scale(&self) -> Vector3<f32> {
        let storage = self.storage.borrow();
        let transform = storage.transform(self.index);
        transform.scale
    }
    pub fn transform(&self) -> Transform {
        let storage = self.storage.borrow();
        storage.transform(self.index)
    }
    pub fn parent_tranform(&self) -> Transform {
        let storage = self.storage.borrow();
        storage.parent_tranform(self.index)
    }
    pub fn info(&self) -> ObjectInfo {
        let storage = self.storage.borrow();
        storage.info(self.index)
    }
    pub fn mesh(&self) -> Option<Mesh> {
        let storage = self.storage.borrow();
        storage.mesh(self.index)
    }
    pub fn set_mesh(&self, mesh: Mesh) {
        let mut storage = self.storage.borrow_mut();
        let m = storage.get_mut_mesh(self.index);
        *m = Some(mesh);
    }
    pub fn set_parent_transform(&self, transform: Transform) {
        let mut storage = self.storage.borrow_mut();
        let t = storage.get_mut_parent_transform(self.index);
        *t = transform;
    }
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn add(&mut self, node: RcRcell<Node>) {
        node.borrow().apply_parent_transform(self.transform());
        self.children.push(node);
    }
    pub fn own(&mut self, node: Node) {
        node.apply_parent_transform(self.transform());
        self.owned_children.push(node);
    }
    pub fn storage(&self) -> RcRcell<Storage> {
        self.storage.clone()
    }
    pub fn children(&self) -> &Vec<RcRcell<Node>> {
        &self.children
    }
    pub fn owned_children(&self) -> &Vec<Node> {
        &self.owned_children
    }
}

/// A Scene tree that facilitates creation of varieties of Nodes; this also owns the Storage.
pub struct Scene {
    root: Node,
    renderer: RcRcell<Renderer>,
}

impl Scene {
    pub fn new(renderer: RcRcell<Renderer>) -> Self {
        let storage = rc_rcell(Storage::new());
        let root = {
            Self::object_default(storage, &renderer.borrow(), "Scene")
        };
        Self {
            root,
            renderer,
        }
    }
    pub fn add(&mut self, node: RcRcell<Node>) {
        self.add_with_mode(node, DrawMode::Triangle);
    }
    pub fn show(&self, node: &Node, mode: DrawMode) {
        {
            let sto = node.storage();
            let mut storage = sto.borrow_mut();
            let info = storage.get_mut_info(node.index());
            info.draw_mode = mode;
        }
        for child in node.children() {
            let child = child.borrow();
            self.show(&child, mode);
        }
        for child in node.owned_children() {
            self.show(child, mode);
        }
    }
    pub fn add_with_mode(&mut self, node: RcRcell<Node>, mode: DrawMode) {
        self.root.add(node.clone());
        self.show(&node.borrow(), mode);
    }
    fn object_default(storage: RcRcell<Storage>, renderer: &Renderer, name: &str) -> Node {
        Self::object(storage, renderer, None, Default::default(), name)
    }
    fn object(
        storage: RcRcell<Storage>,
        renderer: &Renderer,
        mesh: Option<Mesh>,
        transform: Transform,
        name: &str,
    ) -> Node {
        let sto = storage.clone();
        let mut a_storage = sto.borrow_mut();
        let vao = renderer.create_vao(&mesh);
        let index = a_storage.add(mesh, vao, transform, name.into());
        Node {
            index,
            storage,
            children: Vec::new(),
            owned_children: Vec::new(),
        }
    }
    pub fn empty_w_name(&self, name: &str) -> Node {
        Self::object_default(self.storage(), &self.renderer.borrow(), name)
    }
    pub fn empty(&self) -> Node {
        self.empty_w_name("Empty")
    }
    pub fn root(&self) -> &Node {
        &self.root
    }
    pub fn object_from_mesh(&self, geometry: Geometry, material: Material) -> Node {
        Self::object(
            self.storage(),
            &self.renderer.borrow(),
            Some(Mesh { geometry, material }),
            Default::default(),
            "node",
        )
    }
    pub fn object_from_mesh_and_name(
        &self,
        geometry: Geometry,
        material: Material,
        name: &str,
    ) -> Node {
        Self::object(
            self.storage(),
            &self.renderer.borrow(),
            Some(Mesh { geometry, material }),
            Default::default(),
            name,
        )
    }
    pub fn storage(&self) -> RcRcell<Storage> {
        self.root.storage()
    }
    pub fn duplicate_node(&self, node: &Node) -> Node {
        let transform = node.transform();
        let info = node.info();
        let mesh = node.mesh();
        Self::object(self.storage(), &self.renderer.borrow(), mesh, transform, info.name.as_str())
    }
}

/// The main data structure that holds almost everything: object info, meshes, transforms, vaos,
/// etc.
#[derive(Debug, Clone, PartialEq)]
pub struct Storage {
    info: Vec<ObjectInfo>,
    meshes: Vec<Option<Mesh>>,
    transforms: Vec<Transform>,
    parent_transforms: Vec<Transform>,
    vaos: Vec<Option<WebGlVertexArrayObject>>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            info: Vec::new(),
            meshes: Vec::new(),
            transforms: Vec::new(),
            parent_transforms: Vec::new(),
            vaos: Vec::new(),
        }
    }
    pub fn add(&mut self, mesh: Option<Mesh>, vao: Option<WebGlVertexArrayObject>, transform: Transform, name: &str) -> usize {
        let index = self.meshes.len();
        self.meshes.push(mesh);
        self.transforms.push(transform);
        self.parent_transforms.push(Default::default());
        self.vaos.push(vao);
        self.info.push(ObjectInfo {
            name: String::from(name),
            draw_mode: DrawMode::None,
        });
        index
    }
    pub fn setup_vao(&mut self) {}
    pub fn mut_transform(&mut self, indx: usize) -> &mut Transform {
        self.transforms
            .get_mut(indx)
            .expect("No such transform found!")
    }
    pub fn transform(&self, indx: usize) -> Transform {
        *self.transforms.get(indx).expect("No such transform found!")
    }
    pub fn get_mut_parent_transform(&mut self, indx: usize) -> &mut Transform {
        self.parent_transforms
            .get_mut(indx)
            .expect("No such transform found!")
    }
    pub fn parent_tranform(&self, indx: usize) -> Transform {
        *self
            .parent_transforms
            .get(indx)
            .expect("No such transform found!")
    }
    pub fn mesh(&self, indx: usize) -> Option<Mesh> {
        self.meshes.get(indx).expect("No such mesh found!").clone()
    }
    pub fn get_mut_mesh(&mut self, indx: usize) -> &mut Option<Mesh> {
        self.meshes.get_mut(indx).expect("No such mesh found!")
    }
    pub fn meshes(&self) -> &Vec<Option<Mesh>> {
        &self.meshes
    }
    pub fn info(&self, indx: usize) -> ObjectInfo {
        self.info.get(indx).expect("No node info found!").clone()
    }
    pub fn get_vao(&self, indx: usize) -> Option<&WebGlVertexArrayObject> {
        self.vaos.get(indx).expect("No vao info found!").as_ref()
    }
    pub fn get_mut_info(&mut self, indx: usize) -> &mut ObjectInfo {
        self.info.get_mut(indx).expect("No node info found!")
    }
}
