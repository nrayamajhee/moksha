use crate::{
    mesh::multiply, renderer::ShaderType, Color, Mesh, ObjectInfo, RcRcell, Storage, Transform,
};
use nalgebra::{Isometry3, Point3, UnitQuaternion, Vector3};
use ncollide3d::{query::Ray, query::RayCast, shape::ConvexHull};


/// An entity in the scene that holds reference to its props in Storage, keeps tracks of
/// other nodes that are its children either borrowed or owned.
#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    index: usize,
    storage: RcRcell<Storage>,
    children: Vec<RcRcell<Object>>,
    owned_children: Vec<Object>,
}

impl Object {
    pub fn new(index: usize, storage: RcRcell<Storage>) -> Self {
        Object {
            index,
            storage,
            children: Vec::new(),
            owned_children: Vec::new(),
        }
    }
    pub fn position(&self) -> Point3<f32> {
        let transform = self.transform();
        let v = transform.isometry.translation.vector;
        Point3::new(v.x, v.y, v.z)
    }
    pub fn global_position(&self) -> [f32; 3] {
        let transform = self.transform();
        let p_transform = self.parent_transform();
        let v = (p_transform * transform).isometry.translation.vector;
        [v.x, v.y, v.z]
    }
    pub fn rotation(&self) -> UnitQuaternion<f32> {
        let transform = self.transform();
        transform.isometry.rotation
    }
    pub fn scale(&self) -> Vector3<f32> {
        let transform = self.transform();
        transform.scale
    }
    pub fn set_position(&self, x: f32, y: f32, z: f32) {
        let p_transform = {
            let mut storage = self.storage.borrow_mut();
            let transform = storage.mut_transform(self.index);
            transform.isometry.translation.vector = Vector3::new(x, y, z);
            *transform
        };
        self.apply_parent_transform(self.parent_transform() * p_transform);
    }
    pub fn copy_location(&self, object: &Object) {
        let v = object.global_position();
        self.set_position(v[0], v[1], v[2]);
    }
    pub fn set_rotation(&self, rot: UnitQuaternion<f32>) {
        let p_transform = {
            let mut storage = self.storage.borrow_mut();
            let mut transform = storage.mut_transform(self.index);
            transform.isometry.rotation = rot;
            *transform
        };
        self.apply_parent_transform(self.parent_transform() * p_transform);
    }
    pub fn rotate_by(&self, rot: UnitQuaternion<f32>) {
        let p_transform = {
            let mut storage = self.storage.borrow_mut();
            let transform = storage.mut_transform(self.index);
            transform.isometry.append_rotation_wrt_center_mut(&rot);
            *transform
        };
        self.apply_parent_transform(self.parent_transform() * p_transform);
    }
    pub fn set_scale(&self, scale: f32) {
        self.set_scale_vec(scale, scale, scale);
    }
    pub fn set_scale_vec(&self, x: f32, y: f32, z: f32) {
        let p_transform = {
            let mut storage = self.storage.borrow_mut();
            let transform = storage.mut_transform(self.index);
            transform.scale = Vector3::new(x, y, z);
            *transform
        };
        self.apply_parent_transform(self.parent_transform() * p_transform);
    }
    pub fn apply_parent_transform(&self, transform: Transform) {
        let apply_transform = |child: &Object, t: Transform| {
            child.set_parent_transform(t);
            let p_t = t * child.transform();
            child.apply_parent_transform(p_t);
        };
        for child in self.children.iter() {
            apply_transform(&child.borrow(), transform);
        }
        for child in self.owned_children.iter() {
            apply_transform(&child, transform);
        }
    }
    pub fn transform(&self) -> Transform {
        let storage = self.storage.borrow();
        storage.transform(self.index)
    }
    pub fn parent_transform(&self) -> Transform {
        let storage = self.storage.borrow();
        storage.parent_transform(self.index)
    }
    pub fn set_transform(&self, transform: Transform) {
        let p_transform = {
            let mut storage = self.storage.borrow_mut();
            let t = storage.mut_transform(self.index);
            *t = transform;
            *t
        };
        self.apply_parent_transform(p_transform);
    }
    pub fn set_parent_transform(&self, transform: Transform) {
        let mut storage = self.storage.borrow_mut();
        let t = storage.mut_parent_transform(self.index);
        *t = transform;
    }
    pub fn info(&self) -> ObjectInfo {
        let storage = self.storage.borrow();
        storage.info(self.index)
    }
    pub fn set_info(&self, info: ObjectInfo) {
        let mut storage = self.storage.borrow_mut();
        *storage.mut_info(self.index) = info;
    }
    pub fn mesh(&self) -> Option<Mesh> {
        let storage = self.storage.borrow();
        storage.mesh(self.index)
    }
    pub fn set_mesh(&self, mesh: Option<Mesh>) {
        let mut storage = self.storage.borrow_mut();
        let m = storage.mut_mesh(self.index);
        *m = mesh;
    }
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn add(&mut self, object: RcRcell<Object>) {
        self.children.push(object);
        self.children.sort_by_cached_key(|e| e.borrow().info().name);
        self.apply_parent_transform(self.parent_transform() * self.transform());
    }
    pub fn find_child(&self, name: &str) -> Option<usize> {
        if let Ok(i) = self
            .children
            .binary_search_by_key(&String::from(name), |a| a.borrow().info().name)
        {
            Some(i)
        } else {
            None
        }
    }
    pub fn remove(&mut self, name: &str) {
        if let Some(i) = self.find_child(name) {
            self.children.remove(i);
        }
        self.apply_parent_transform(Transform::identity());
    }
    pub fn own(&mut self, object: Object) {
        self.owned_children.push(object);
        self.apply_parent_transform(self.parent_transform() * self.transform());
    }
    pub fn storage(&self) -> RcRcell<Storage> {
        self.storage.clone()
    }
    pub fn children(&self) -> &Vec<RcRcell<Object>> {
        &self.children
    }
    pub fn owned_children(&self) -> &Vec<Object> {
        &self.owned_children
    }
    pub fn collies_w_owned_children_recursive(&self, ray: &Ray<f32>) -> Option<Isometry3<f32>> {
        if let Some(t) = self.collides_w_ray(ray) {
            return Some(t);
        }
        for child in self.owned_children() {
            if let Some(t) = child.collies_w_owned_children_recursive(ray) {
                return Some(t);
            }
        }
        None
    }
    fn collides_w_children_recursive(
        ray: &Ray<f32>,
        object: RcRcell<Object>,
    ) -> Option<(RcRcell<Object>, Isometry3<f32>)> {
        if let Some(t) = object.borrow().collides_w_ray(ray) {
            return Some((object.clone(), t));
        }
        for child in object.borrow().children() {
            if let Some(result) = Self::collides_w_children_recursive(ray, child.clone()) {
                return Some(result);
            }
        }
        for child in object.borrow().owned_children() {
            if let Some(t) = child.collies_w_owned_children_recursive(ray) {
                return Some((object.clone(), t));
            }
        }
        None
    }
    pub fn collides_w_children(&self, ray: &Ray<f32>) -> Option<(RcRcell<Object>, Isometry3<f32>)> {
        for each in self.children() {
            if let Some(result) = Self::collides_w_children_recursive(&ray, each.clone()) {
                return Some(result);
            }
        }
        None
    }
    pub fn collides_w_ray(&self, ray: &Ray<f32>) -> Option<Isometry3<f32>> {
        let t = self.transform();
        let p_t = self.parent_transform();
        let s = multiply(t.scale, p_t.scale);
        if let Some(mesh) = self.mesh() {
            let verts: Vec<Point3<f32>> = mesh
                .geometry
                .vertices
                .chunks(3)
                .map(|c| Point3::new(c[0] as f32 * s.x, c[1] as f32 * s.y, c[2] as f32 * s.z))
                .collect();
            if let Some(target) = ConvexHull::try_from_points(&verts) {
                let transform = (p_t * t).isometry;
                if target.intersects_ray(&transform, &ray) {
                    return Some(transform);
                }
            }
        }
        None
    }
    pub fn change_color(&self, color: [f32; 3]) {
        let mut mesh = self.mesh().unwrap();
        mesh.material = mesh.material.color(color[0], color[1], color[2], 1.);
        self.set_mesh(Some(mesh));
    }
    pub fn set_outline(&self, outline_scale: Option<f32>) {
        if let Some(mut mesh) = self.mesh() {
            mesh.material.outline = outline_scale;
            self.set_mesh(Some(mesh));
        }
        for each in self.owned_children() {
            each.set_outline(outline_scale);
        }
    }
}
