use crate::{scene::LightInfo, Mesh, ObjectInfo, Transform};
use std::rc::Rc;
use web_sys::{WebGlTexture, WebGlVertexArrayObject};

/// The main data structure that holds almost everything: object info, meshes, transforms, vaos,
/// etc.
#[derive(Debug, Clone, PartialEq)]
pub struct Storage {
    info: Vec<ObjectInfo>,
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
            transforms: Vec::new(),
            parent_transforms: Vec::new(),
            vaos: Vec::new(),
            textures: Vec::new(),
            lights: Vec::new(),
        }
    }
}

impl Storage {
    pub fn add_light(&mut self, light: LightInfo) -> usize {
        let index = self.lights.len();
        self.lights.push(light);
        index
    }
    pub fn add_texture(&mut self, texture: Rc<WebGlTexture>) -> usize {
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
    ) -> usize {
        let index = self.meshes.len();
        self.meshes.push(mesh);
        self.transforms.push(transform);
        self.parent_transforms.push(Default::default());
        self.vaos.push(vao);
        self.info.push(info);
        index
    }
    pub fn mut_transform(&mut self, indx: usize) -> &mut Transform {
        self.transforms
            .get_mut(indx)
            .expect("No such transform found!")
    }
    pub fn transform(&self, indx: usize) -> Transform {
        *self.transforms.get(indx).expect("No such transform found!")
    }
    pub fn parent_transform(&self, indx: usize) -> Transform {
        *self
            .parent_transforms
            .get(indx)
            .expect("No such transform found!")
    }
    pub fn mut_parent_transform(&mut self, indx: usize) -> &mut Transform {
        self.parent_transforms
            .get_mut(indx)
            .expect("No such transform found!")
    }
    pub fn mesh(&self, indx: usize) -> Option<Mesh> {
        self.meshes.get(indx).expect("No such mesh found!").clone()
    }
    pub fn texture(&self, indx: usize) -> &WebGlTexture {
        &self.textures.get(indx).expect("No such texture found!")
    }
    pub fn mut_mesh(&mut self, indx: usize) -> &mut Option<Mesh> {
        self.meshes.get_mut(indx).expect("No such mesh found!")
    }
    pub fn meshes(&self) -> &Vec<Option<Mesh>> {
        &self.meshes
    }
    pub fn info(&self, indx: usize) -> ObjectInfo {
        self.info.get(indx).expect("No node info found!").clone()
    }
    pub fn vao(&self, indx: usize) -> Option<&WebGlVertexArrayObject> {
        self.vaos.get(indx).expect("No vao info found!").as_ref()
    }
    pub fn mut_info(&mut self, indx: usize) -> &mut ObjectInfo {
        self.info.get_mut(indx).expect("No node info found!")
    }
    pub fn light(&self, indx: usize) -> LightInfo {
        *self.lights.get(indx).expect("No light info found!")
    }
    pub fn lights(&self) -> &Vec<LightInfo> {
        &self.lights
    }
    pub fn mut_light_info(&mut self, indx: usize) -> &mut LightInfo {
        self.lights.get_mut(indx).expect("No node info found!")
    }
}
