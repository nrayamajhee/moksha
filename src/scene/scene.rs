use crate::{
    Geometry, Material, Mesh, Transform, Node, Storage,
    rc_rcell,
    renderer::{DrawMode, Renderer},
    RcRcell,
};

#[derive(Debug)]
pub enum LightType {
    Ambient,
    Point,
    Directional,
    Spot
}

#[allow(dead_code)]
pub struct Light {
    light_type: LightType,
    color: [f32;3],
    intensity: f32,
}


#[derive(Debug, Clone, PartialEq)]
pub struct ObjectInfo {
    pub name: String,
    pub draw_mode: DrawMode,
    pub render: bool,
}

impl Default for ObjectInfo {
    fn default() -> Self {
        Self {
            name: "node".into(),
            draw_mode: DrawMode::Triangle,
            render: false,
        }
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
        let root = Self::object(
            storage,
            &renderer.borrow(),
            None,
            Default::default(),
            ObjectInfo {
                name: "Scene".into(),
                ..Default::default()
            });
        Self { root, renderer }
    }
    pub fn root(&self) -> &Node {
        &self.root
    }
    pub fn show(&self, node: &Node) {
        {
            let s = self.storage();
            let mut storage = s.borrow_mut();
            let mut info = storage.mut_info(node.index());
            info.render = true;
        }
        for child in node.children() {
            let child = child.borrow();
            self.show(&child);
        }
        for child in node.owned_children() {
            self.show(child);
        }
    }
    pub fn add(&mut self, node: RcRcell<Node>) {
        self.show(&node.borrow());
        self.root.add(node);
    }
    fn object(
        storage: RcRcell<Storage>,
        renderer: &Renderer,
        mesh: Option<Mesh>,
        transform: Transform,
        info: ObjectInfo,
    ) -> Node {
        let sto = storage.clone();
        let mut a_storage = sto.borrow_mut();
        let vao = renderer.create_vao(&mesh);
        let index = a_storage.add(mesh, vao, transform, info);
        Node::new(index, storage)
    }
    pub fn empty(&self) -> Node {
        self.empty_w_name("Empty")
    }
    pub fn empty_w_name(&self, name: &str) -> Node {
        Self::object(self.storage(), &self.renderer.borrow(), None, Default::default(), ObjectInfo {
            name: name.into(),
            ..Default::default()
        })
    }
    pub fn object_from_mesh_and_info(
        &self,
        mesh: Mesh,
        info: ObjectInfo,
    ) -> Node {
        Self::object(
            self.storage(),
            &self.renderer.borrow(),
            Some(mesh),
            Default::default(),
            info
        )
    }
    pub fn object_from_mesh_name_and_mode(
        &self,
        geometry: Geometry,
        material: Material,
        name: &str,
        draw_mode: DrawMode,
    ) -> Node {
        self.object_from_mesh_and_info(Mesh{geometry, material}, ObjectInfo{name:name.into(), draw_mode,..Default::default()})
    }
    pub fn object_from_mesh_and_name(
        &self,
        geometry: Geometry,
        material: Material,
        name: &str,
    ) -> Node {
        self.object_from_mesh_and_info(Mesh{geometry, material}, ObjectInfo{name:name.into(),..Default::default()})
    }
    pub fn object_from_mesh(&self, geometry: Geometry, material: Material) -> Node {
        self.object_from_mesh_and_name(geometry, material, "node")
    }
    //pub fn light(&self, light_type: LightType) -> Node {
        //let (geometry, material) = match light_type {
            //LightType::Ambient => (
                //Geometry::from_genmesh_no_normals(&IcoSphere::subdivide(2)),
                //Material::single_color_no_shade(1.0, 1.0, 0.0, 1.0),
            //),
            //LightType::Directional => (
            //),
            //LightType::Point => (
            //),
            //LightType::Spot => (
            //),
        //};
        //self.object_from_mesh_and_name(geometry, material, light_type.to_string().as_str())
    //}
    pub fn storage(&self) -> RcRcell<Storage> {
        self.root.storage()
    }
    pub fn duplicate_node(&self, node: &Node) -> Node {
        let transform = node.transform();
        let info = node.info();
        let mesh = node.mesh();
        Self::object(
            self.storage(),
            &self.renderer.borrow(),
            mesh,
            transform,
            info,
        )
    }
}
