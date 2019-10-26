use crate::{Mesh, ObjectInfo, RcRcell, Storage, Transform, mesh::multiply, Material};
use nalgebra::{Point3, UnitQuaternion, Vector3, Isometry3};
use ncollide3d::{query::Ray, shape::ConvexHull, query::RayCast};

/// An entity in the scene that holds reference to its props in Storage, keeps tracks of
/// other nodes that are its children either borrowed or owned.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    index: usize,
    storage: RcRcell<Storage>,
    children: Vec<RcRcell<Node>>,
    owned_children: Vec<Node>,
}

impl Node {
    pub fn new(index: usize, storage: RcRcell<Storage>) -> Self {
        Node {
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
    pub fn copy_location(&self, node: &Node) {
        let v = node.global_position();
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
        let apply_transform = |child: &Node, t: Transform| {
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
    pub fn set_transform(&self, transform: Transform) {
        let p_transform = {
            let mut storage = self.storage.borrow_mut();
            let t = storage.mut_transform(self.index);
            *t = transform;
            *t
        };
        self.apply_parent_transform(p_transform);
    }
    pub fn parent_transform(&self) -> Transform {
        let storage = self.storage.borrow();
        storage.parent_tranform(self.index)
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
    pub fn mesh(&self) -> Option<Mesh> {
        let storage = self.storage.borrow();
        storage.mesh(self.index)
    }
    pub fn set_mesh(&self, mesh: Mesh) {
        let mut storage = self.storage.borrow_mut();
        let m = storage.mut_mesh(self.index);
        *m = Some(mesh);
    }
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn add(&mut self, node: RcRcell<Node>) {
        self.children.push(node);
        self.apply_parent_transform(self.parent_transform() * self.transform());
    }
    pub fn own(&mut self, node: Node) {
        self.owned_children.push(node);
        self.apply_parent_transform(self.parent_transform() * self.transform());
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
    pub fn owned_children_collide_w_ray(&self, ray: &Ray<f32>) -> Option<Isometry3<f32>> {
        for child in self.owned_children() {
            if let Some(t) = child.collides_w_ray(ray) {
                return Some(t);
            }
        }
        None
    }
    fn collides_w_children_recursive(ray: &Ray<f32>, node: RcRcell<Node>) -> Option<(RcRcell<Node>, Isometry3<f32>)> {
        if let Some(t) = node.borrow().collides_w_ray(ray) {
            return Some((node.clone(), t));
        }
        for child in node.borrow().children() {
            if let Some(result) = Self::collides_w_children_recursive(ray, child.clone()) {
                return Some(result);
            }
        }
        if let Some(t) = node.borrow().owned_children_collide_w_ray(ray) {
            return Some((node.clone(),t));
        }
        None
    }
    pub fn collides_w_children(&self, ray: &Ray<f32>) -> Option<(RcRcell<Node>, Isometry3<f32>)> {
        for each in self.children() {
            if let Some(result) =
                Self::collides_w_children_recursive(&ray, each.clone())
            {
                return Some(result);
            }
        }
        None
    }
    pub fn collides_w_ray(&self, ray: &Ray<f32>) -> Option<Isometry3<f32>>{
        let t = self.transform();
        let p_t = self.parent_transform();
        let s = multiply(t.scale, p_t.scale);
        if let Some(mesh) = self.mesh() {
            let verts: Vec<Point3<f32>> = mesh
                .geometry
                .vertices
                .chunks(3)
                .map(|c| Point3::new(c[0] * s.x, c[1] * s.y, c[2] * s.z))
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
        mesh.material = Material::single_color_no_shade(color[0], color[1], color[2], 1.);
        self.set_mesh(mesh);
    }
}
