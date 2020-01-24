use super::{LightInfo, ObjectInfo};
use crate::{Mesh, Transform};
use std::rc::Rc;
use web_sys::{WebGlTexture, WebGlVertexArrayObject};

pub type Id = usize;

/// The main data structure that holds almost everything: object info, meshes, transforms, vaos,
/// etc.
#[derive(Debug, Clone, PartialEq)]
pub struct Storage {
    info: Vec<ObjectInfo>,
    children: Vec<Vec<Id>>,
    parent: Vec<Option<Id>>,
    meshes: Vec<Option<Mesh>>,
    transforms: Vec<Transform>,
    parent_transforms: Vec<Transform>,
    vaos: Vec<Option<WebGlVertexArrayObject>>,
    textures: Vec<Rc<WebGlTexture>>,
    lights: Vec<LightInfo>,
}

impl Default for Storage {
    fn default() -> Self {
        Self {
            info: Vec::new(),
            meshes: Vec::new(),
            children: Vec::new(),
            parent: Vec::new(),
            transforms: Vec::new(),
            parent_transforms: Vec::new(),
            vaos: Vec::new(),
            textures: Vec::new(),
            lights: Vec::new(),
        }
    }
}

impl Storage {
    pub fn add_light(&mut self, light: LightInfo) -> Id {
        let index = self.lights.len();
        self.lights.push(light);
        index
    }
    pub fn add_texture(&mut self, texture: Rc<WebGlTexture>) -> Id {
        let index = self.textures.len();
        self.textures.push(texture);
        index
    }
    pub fn add(
        &mut self,
        mesh: Option<Mesh>,
        vao: Option<WebGlVertexArrayObject>,
        transform: Transform,
        info: ObjectInfo,
    ) -> Id {
        let index = self.meshes.len();
        self.meshes.push(mesh);
        self.transforms.push(transform);
        self.parent_transforms.push(Default::default());
        self.children.push(Vec::new());
        self.vaos.push(vao);
        self.info.push(info);
        index
    }
    pub fn mut_transform(&mut self, indx: Id) -> &mut Transform {
        self.transforms
            .get_mut(indx)
            .expect("No such transform found!")
    }
    pub fn transform(&self, indx: Id) -> Transform {
        *self.transforms.get(indx).expect("No such transform found!")
    }
    pub fn parent_transform(&self, indx: Id) -> Transform {
        *self
            .parent_transforms
            .get(indx)
            .expect("No such transform found!")
    }
    pub fn mut_parent_transform(&mut self, indx: Id) -> &mut Transform {
        self.parent_transforms
            .get_mut(indx)
            .expect("No such transform found!")
    }
    pub fn mesh(&self, indx: Id) -> Option<Mesh> {
        self.meshes.get(indx).expect("No such mesh found!").clone()
    }
    pub fn texture(&self, indx: Id) -> &WebGlTexture {
        &self.textures.get(indx).expect("No such texture found!")
    }
    pub fn mut_mesh(&mut self, indx: Id) -> &mut Option<Mesh> {
        self.meshes.get_mut(indx).expect("No such mesh found!")
    }
    pub fn meshes(&self) -> &Vec<Option<Mesh>> {
        &self.meshes
    }
    pub fn has_object(&self, index: Id) -> bool {
        if let Some(index) = self.info.get(index) {
            true
        } else {
            false
        }
    }
    pub fn info(&self, indx: Id) -> ObjectInfo {
        self.info.get(indx).expect("No object info found!").clone()
    }
    pub fn vao(&self, indx: Id) -> Option<&WebGlVertexArrayObject> {
        self.vaos.get(indx).expect("No vao info found!").as_ref()
    }
    pub fn mut_info(&mut self, indx: Id) -> &mut ObjectInfo {
        self.info.get_mut(indx).expect("No object info found!")
    }
    pub fn light(&self, indx: Id) -> LightInfo {
        *self.lights.get(indx).expect("No light info found!")
    }
    pub fn lights(&self) -> &Vec<LightInfo> {
        &self.lights
    }
    pub fn mut_light_info(&mut self, indx: Id) -> &mut LightInfo {
        self.lights.get_mut(indx).expect("No object info found!")
    }
    pub fn children(&self, index: Id) -> &Vec<Id> {
        &self.children.get(index).expect("No object found!")
    }
    pub fn mut_children(&mut self, index: Id) -> &mut Vec<Id> {
        self.children.get_mut(index).expect("No object found!")
    }
    pub fn parent(&self, index: Id) -> Option<Id> {
        *self.parent.get(index).expect("No object found!")
    }
}
