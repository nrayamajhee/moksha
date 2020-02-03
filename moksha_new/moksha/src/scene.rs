use crate::{
    mesh::Mesh,
    mesh::Transform,
    RcRcell,
    rc_rcell,
    storage::{Component, Entity, Id, Storage},
};

#[derive(Debug)]
pub struct Node {
    pub name: String,
    pub children: Vec<Id>,
    pub parent: Option<Id>,
}

impl Component for Node {}

#[derive(Debug)]
pub struct Scene {
    root: Entity,
}

impl Scene {
    pub fn new() -> Self {
        let storage = rc_rcell(Storage::new());
        let root = Entity::new(storage.clone()).with(
            "node".into(),
            Node {
                name: "root".into(),
                children: Vec::new(),
                parent: None,
            },
        );
        Self { root }
    }
    pub fn storage(&self) -> RcRcell<Storage> {
        self.root.storage.clone()
    }
    pub fn empty(&self, name: &str) -> Entity {
        let node = Node {
            name: name.into(),
            children: Vec::new(),
            parent: None,
        };
        let transform = Transform::identity();
        Entity::new(self.storage())
            .with("node", node)
            .with("transfrom", transform)
    }
    pub fn mesh(&self, name: &str, mesh: Mesh) -> Entity {
        Entity::new(self.storage())
            .with(
                "node",
                Node {
                    name: name.into(),
                    children: Vec::new(),
                    parent: None,
                },
            )
            .with("transfrom", Transform::identity())
            .with("mesh", mesh)
    }
}
