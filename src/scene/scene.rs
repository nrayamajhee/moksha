use crate::mesh::{Geometry, Material, Mesh, Transform};
use crate::renderer::DrawMode;
// use genmesh::generators::{Cone, Cylinder, IcoSphere};
use nalgebra::{UnitQuaternion, Vector3};
use std::cell::RefCell;
// use std::f32::consts::PI;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct ObjectInfo {
    pub name: String,
    pub draw_mode: DrawMode,
}

#[derive(Debug)]
pub struct Node {
    index: usize,
    storage: Rc<RefCell<Storage>>,
    children: Vec<Rc<RefCell<Node>>>,
    owned_children: Vec<Node>,
}

impl Node {
    pub fn set_position(&self, pos: [f32; 3]) {
        let p_transform = {
            let mut storage = self.storage.borrow_mut();
            let transform = storage.get_mut_transform(self.index);
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
    pub fn scale(&self, scale: f32) {
        let p_transform = {
            let mut storage = self.storage.borrow_mut();
            let transform = storage.get_mut_transform(self.index);
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
        let p_transform = transform * self.get_transform();
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
            let mut transform = storage.get_mut_transform(self.index);
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
            let transform = storage.get_mut_transform(self.index);
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
            let t = storage.get_mut_transform(self.index);
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
    pub fn scale_vec(&self, scale: [f32; 3]) {
        let mut storage = self.storage.borrow_mut();
        let transform = storage.get_mut_transform(self.index);
        transform.scale = Vector3::new(scale[0], scale[2], scale[1]);
    }
    pub fn get_scale(&self) -> Vector3<f32> {
        let mut storage = self.storage.borrow_mut();
        let transform = storage.get_mut_transform(self.index);
        transform.scale
    }
    pub fn get_transform(&self) -> Transform {
        let storage = self.storage.borrow_mut();
        storage.get_transform(self.index)
    }
    pub fn get_parent_transform(&self) -> Transform {
        let storage = self.storage.borrow_mut();
        storage.get_parent_transform(self.index)
    }
    pub fn get_info(&self) -> ObjectInfo {
        let storage = self.storage.borrow();
        storage.get_info(self.index)
    }
    pub fn get_mesh(&self) -> Option<Mesh> {
        let storage = self.storage.borrow();
        storage.get_mesh(self.index)
    }
    pub fn set_parent_transform(&self, transform: Transform) {
        let mut storage = self.storage.borrow_mut();
        let t = storage.get_mut_parent_transform(self.index);
        *t = transform;
    }
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn add(&mut self, node: Rc<RefCell<Node>>) {
        node.borrow().apply_parent_transform(self.get_transform());
        self.children.push(node);
    }
    pub fn own(&mut self, node: Node) {
        node.apply_parent_transform(self.get_transform());
        self.owned_children.push(node);
    }
    pub fn get_storage(&self) -> Rc<RefCell<Storage>> {
        self.storage.clone()
    }
    pub fn children(&self) -> &Vec<Rc<RefCell<Node>>> {
        &self.children
    }
    pub fn owned_children(&self) -> &Vec<Node> {
        &self.owned_children
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
    pub fn add(&self, node: &Node) {
        self.add_with_mode(node, DrawMode::Triangle);
    }
    pub fn add_with_mode(&self, node: &Node, mode: DrawMode) {
        {
            let sto = node.get_storage();
            let mut storage = sto.borrow_mut();
            let info = storage.get_mut_info(node.index());
            info.draw_mode = mode;
        }
        for child in node.children() {
            let child = child.borrow();
            self.add_with_mode(&child, mode);
        }
        for child in node.owned_children() {
            self.add_with_mode(child, mode);
        }
    }
    fn object_default(storage: Rc<RefCell<Storage>>) -> Node {
        Self::object(storage, None, Default::default(), "node")
    }
    fn object(
        storage: Rc<RefCell<Storage>>,
        mesh: Option<Mesh>,
        transform: Transform,
        name: &str,
    ) -> Node {
        let sto = storage.clone();
        let mut a_storage = sto.borrow_mut();
        let index = a_storage.add(mesh, transform, name.into());
        Node {
            index,
            storage,
            children: Vec::new(),
            owned_children: Vec::new(),
        }
    }
    pub fn empty(&self) -> Node {
        Self::object_default(self.get_storage())
    }
    pub fn object_from_mesh(&self, geometry: Geometry, material: Material) -> Node {
        Self::object(
            self.get_storage(),
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
            self.get_storage(),
            Some(Mesh { geometry, material }),
            Default::default(),
            name,
        )
    }
    pub fn get_storage(&self) -> Rc<RefCell<Storage>> {
        self.root.get_storage()
    }
    pub fn duplicate_node(&self, node: &Node) -> Node {
        let transform = node.get_transform();
        let info = node.get_info();
        let mesh = node.get_mesh();
        Self::object(self.get_storage(), mesh, transform, info.name.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct Storage {
    info: Vec<ObjectInfo>,
    meshes: Vec<Option<Mesh>>,
    transforms: Vec<Transform>,
    parent_transforms: Vec<Transform>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            info: Vec::new(),
            meshes: Vec::new(),
            transforms: Vec::new(),
            parent_transforms: Vec::new(),
        }
    }
    pub fn add(&mut self, mesh: Option<Mesh>, transform: Transform, name: &str) -> usize {
        let index = self.meshes.len();
        self.meshes.push(mesh);
        self.transforms.push(transform);
        self.parent_transforms.push(Default::default());
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
    pub fn get_mut_parent_transform(&mut self, indx: usize) -> &mut Transform {
        self.parent_transforms
            .get_mut(indx)
            .expect("No such transform found!")
    }
    pub fn get_parent_transform(&self, indx: usize) -> Transform {
        *self
            .parent_transforms
            .get(indx)
            .expect("No such transform found!")
    }
    pub fn get_mesh(&self, indx: usize) -> Option<Mesh> {
        self.meshes.get(indx).expect("No such mesh found!").clone()
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
