//#![feature(type_alias_impl_trait)]
//#![feature(associated_type_defaults)]
use crate::{
    mesh::multiply, renderer::ShaderType, Color, Id, Mesh, ObjectInfo, RcRcell, Storage, Transform,
};
use nalgebra::{Isometry3, Point3, UnitQuaternion, Vector3};
use ncollide3d::{query::Ray, query::RayCast, shape::ConvexHull};
use std::rc::Rc;

pub trait Node {
    fn storage(&self) -> RcRcell<Storage>;
    fn obj_id(&self) -> Id;
    fn update_id(&mut self, id: Id);
    fn children(&self) -> Iterator<Item = Self>;
}

pub trait Obj: Node {
    fn transform(&self) -> Transform {
        let storage = self.storage();
        let storage = storage.borrow();
        let index = self.obj_id();
        storage.transform(index)
    }
    fn parent_transform(&self) -> Transform {
        let storage = self.storage();
        let storage = storage.borrow();
        let index = self.obj_id();
        storage.parent_transform(index)
    }
    fn position(&self) -> Point3<f32> {
        let transform = self.transform();
        let v = transform.isometry.translation.vector;
        Point3::new(v.x, v.y, v.z)
    }
    fn global_position(&self) -> [f32; 3] {
        let transform = self.transform();
        let p_transform = self.parent_transform();
        let v = (p_transform * transform).isometry.translation.vector;
        [v.x, v.y, v.z]
    }
    fn rotation(&self) -> UnitQuaternion<f32> {
        let transform = self.transform();
        transform.isometry.rotation
    }
    fn scale(&self) -> Vector3<f32> {
        let transform = self.transform();
        transform.scale
    }
    fn info(&self) -> ObjectInfo {
        let storage = self.storage();
        let storage = storage.borrow();
        let index = self.obj_id();
        storage.info(index)
    }
    fn mesh(&self) -> Option<Mesh> {
        let storage = self.storage();
        let storage = storage.borrow();
        let index = self.obj_id();
        storage.mesh(index)
    }
    fn set_position(&self, x: f32, y: f32, z: f32) {
        let p_transform = {
            let storage = self.storage();
            let mut storage = storage.borrow_mut();
            let index = self.obj_id();
            let transform = storage.mut_transform(index);
            transform.isometry.translation.vector = Vector3::new(x, y, z);
            *transform
        };
        self.apply_parent_transform(self.parent_transform() * p_transform);
    }
    fn copy_location<O>(&self, object: &O)
    where
        O: Obj,
    {
        let v = object.global_position();
        self.set_position(v[0], v[1], v[2]);
    }
    fn set_rotation(&self, rot: UnitQuaternion<f32>) {
        let p_transform = {
            let storage = self.storage();
            let mut storage = storage.borrow_mut();
            let index = self.obj_id();
            let mut transform = storage.mut_transform(index);
            transform.isometry.rotation = rot;
            *transform
        };
        self.apply_parent_transform(self.parent_transform() * p_transform);
    }
    fn rotate_by(&self, rot: UnitQuaternion<f32>) {
        let p_transform = {
            let storage = self.storage();
            let mut storage = storage.borrow_mut();
            let index = self.obj_id();
            let transform = storage.mut_transform(index);
            transform.isometry.append_rotation_wrt_center_mut(&rot);
            *transform
        };
        self.apply_parent_transform(self.parent_transform() * p_transform);
    }
    fn set_scale(&self, scale: f32) {
        self.set_scale_vec(scale, scale, scale);
    }
    fn set_scale_vec(&self, x: f32, y: f32, z: f32) {
        let p_transform = {
            let storage = self.storage();
            let mut storage = storage.borrow_mut();
            let index = self.obj_id();
            let transform = storage.mut_transform(index);
            transform.scale = Vector3::new(x, y, z);
            *transform
        };
        self.apply_parent_transform(self.parent_transform() * p_transform);
    }
    fn set_transform(&self, transform: Transform) {
        let p_transform = {
            let storage = self.storage();
            let mut storage = storage.borrow_mut();
            let index = self.obj_id();
            let t = storage.mut_transform(index);
            *t = transform;
            *t
        };
        self.apply_parent_transform(p_transform);
    }
    fn set_parent_transform(&self, transform: Transform) {
        let storage = self.storage();
        let mut storage = storage.borrow_mut();
        let index = self.obj_id();
        let t = storage.mut_parent_transform(index);
        *t = transform;
    }
    fn set_info(&self, info: ObjectInfo) {
        let storage = self.storage();
        let mut storage = storage.borrow_mut();
        let index = self.obj_id();
        *storage.mut_info(index) = info;
    }
    fn set_mesh(&self, mesh: Option<Mesh>) {
        let storage = self.storage();
        let mut storage = storage.borrow_mut();
        let index = self.obj_id();
        let m = storage.mut_mesh(index);
        *m = mesh;
    }
    fn apply_transform(&self, transform: Transform) {
        self.set_parent_transform(transform);
        let p_t = transform * self.transform();
        self.apply_parent_transform(p_t);
    }
    fn apply_parent_transform(&self, transform: Transform) {
        //for child in self.children() {
            //child.apply_transform(transform);
        //}
    }
    fn add<N>(&self, object: &N)
    where
        N: Node,
    {
        let storage = self.storage();
        let mut sto = storage.borrow_mut();
        let mut children = sto.mut_children(self.obj_id());
        children.push(object.obj_id());
        children.sort_by_cached_key(|id| storage.borrow().info(*id).name);
        self.apply_parent_transform(self.parent_transform() * self.transform());
    }
    fn find_child(&self, name: &str) -> Option<(Id, usize)> {
        let storage = self.storage();
        let storage = storage.borrow();
        let children = storage.children(self.obj_id());
        if let Ok(i) =
            children.binary_search_by_key(&String::from(name), |id| storage.info(*id).name)
        {
            Some((children[i], i))
        } else {
            None
        }
    }
    fn remove(&self, name: &str) {
        let storage = self.storage();
        let mut storage = storage.borrow_mut();
        let mut children = storage.mut_children(self.obj_id());
        if let Some((_, i)) = self.find_child(name) {
            children.remove(i);
        }
        self.apply_parent_transform(Transform::identity());
    }
    fn change_color(&self, color: Color) {
        let mut mesh = self.mesh().unwrap();
        mesh.material = mesh.material.color(color.r, color.g, color.b, color.a);
        self.set_mesh(Some(mesh));
    }
    fn set_outline(&self, outline_scale: Option<f32>) {
        if let Some(mut mesh) = self.mesh() {
            mesh.material.outline = outline_scale;
            self.set_mesh(Some(mesh));
        }
        //for each in self.children() {
            //each.set_outline(outline_scale);
        //}
    }
    //fn collies_w_children_recursive(&self, ray: &Ray<f32>) -> Option<Isometry3<f32>> {
    //if let Some(t) = self.collides_w_ray(ray) {
    //return Some(t);
    //}
    //for child in self.owned_children() {
    //if let Some(t) = child.collies_w_owned_children_recursive(ray) {
    //return Some(t);
    //}
    //}
    //None
    //}
    //fn collides_w_children_recursive<N>(
    //ray: &Ray<f32>,
    //object: RcRcell<N>,
    //) -> Option<(RcRcell<N>, Isometry3<f32>)>  where N: Node{
    //if let Some(t) = object.borrow().collides_w_ray(ray) {
    //return Some((object.clone(), t));
    //}
    //for child in object.borrow().children() {
    //if let Some(result) = Self::collides_w_children_recursive(ray, child.clone()) {
    //return Some(result);
    //}
    //}
    //for child in object.borrow().owned_children() {
    //if let Some(t) = child.collies_w_owned_children_recursive(ray) {
    //return Some((object.clone(), t));
    //}
    //}
    //None
    //}
    //fn collides_w_children<N>(&self, ray: &Ray<f32>) -> Option<(RcRcell<N>, Isometry3<f32>)> where N: Node {
    //for each in self.children() {
    //if let Some(result) = Self::collides_w_children_recursive(&ray, each.clone()) {
    //return Some(result);
    //}
    //}
    //None
    //}
    //fn collides_w_ray(&self, ray: &Ray<f32>) -> Option<Isometry3<f32>> {
    //let t = self.transform();
    //let p_t = self.parent_transform();
    //let s = multiply(t.scale, p_t.scale);
    //if let Some(mesh) = self.mesh() {
    //let verts: Vec<Point3<f32>> = mesh
    //.geometry
    //.vertices
    //.chunks(3)
    //.map(|c| Point3::new(c[0] as f32 * s.x, c[1] as f32 * s.y, c[2] as f32 * s.z))
    //.collect();
    //if let Some(target) = ConvexHull::try_from_points(&verts) {
    //let transform = (p_t * t).isometry;
    //if target.intersects_ray(&transform, &ray) {
    //return Some(transform);
    //}
    //}
    //}
    //None
    //}
}
