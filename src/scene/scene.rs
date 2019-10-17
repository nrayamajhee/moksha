use crate::{
    dom_factory::{add_event, window},
    rc_rcell,
    renderer::{CursorType, DrawMode, Renderer},
    scene::primitives::create_light_node,
    Geometry, Material, Mesh, MouseButton, Node, RcRcell, Storage, Transform, Viewport,
};
use std::fmt;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{KeyboardEvent, MouseEvent, WheelEvent};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LightType {
    Ambient = 1,
    Point = 2,
    Directional = 3,
    Spot = 4,
}

impl fmt::Display for LightType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LightInfo {
    pub light_type: LightType,
    pub color: [f32; 3],
    pub node_id: usize,
    pub light: bool,
}

pub struct Light {
    light_id: usize,
    node: Rc<Node>,
}

impl Light {
    pub fn node(&self) -> Rc<Node> {
        self.node.clone()
    }
    pub fn index(&self) -> usize {
        self.light_id
    }
}

/// Information about an object in the scene (name, render flag, drawing mode)
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
    viewport: RcRcell<Viewport>,
}

impl Scene {
    pub fn new(renderer: RcRcell<Renderer>, viewport: RcRcell<Viewport>) -> Self {
        let storage = rc_rcell(Storage::new());
        let root = Self::object(
            storage,
            &renderer.borrow(),
            None,
            Default::default(),
            ObjectInfo {
                name: "Scene".into(),
                ..Default::default()
            },
        );
        let scene = Self {
            root,
            renderer,
            viewport,
        };
        scene.add_viewport_events();
        scene
    }
    pub fn root(&self) -> &Node {
        &self.root
    }
    pub fn add_light(&mut self, light: Light) {
        self.add(light.node());
        let s = self.storage();
        let mut storage = s.borrow_mut();
        let mut info = storage.mut_light_info(light.index());
        info.light = true;
    }
    pub fn show(&self, node: &Node) {
        {
            let s = self.storage();
            let mut storage = s.borrow_mut();
            let mut info = storage.mut_info(node.index());
            info.render = true;
        }
        for child in node.children() {
            self.show(&child);
        }
        for child in node.owned_children() {
            self.show(child);
        }
    }
    pub fn add(&mut self, node: Rc<Node>) {
        self.show(&node);
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
    pub fn light(&self, light_type: LightType, color: [f32; 3]) -> Light {
        let node = Rc::new(create_light_node(&self, light_type, color));
        let light_id = self.storage().borrow_mut().add_light(LightInfo {
            light_type,
            color,
            node_id: node.index(),
            light: false,
        });
        Light { light_id, node }
    }
    pub fn empty(&self) -> Node {
        self.empty_w_name("Empty")
    }
    pub fn empty_w_name(&self, name: &str) -> Node {
        Self::object(
            self.storage(),
            &self.renderer.borrow(),
            None,
            Default::default(),
            ObjectInfo {
                name: name.into(),
                ..Default::default()
            },
        )
    }
    pub fn object_from_mesh_and_info(&self, mesh: Mesh, info: ObjectInfo) -> Node {
        Self::object(
            self.storage(),
            &self.renderer.borrow(),
            Some(mesh),
            Default::default(),
            info,
        )
    }
    pub fn object_from_mesh_name_and_mode(
        &self,
        geometry: Geometry,
        material: Material,
        name: &str,
        draw_mode: DrawMode,
    ) -> Node {
        self.object_from_mesh_and_info(
            Mesh { geometry, material },
            ObjectInfo {
                name: name.into(),
                draw_mode,
                ..Default::default()
            },
        )
    }
    pub fn object_from_mesh_and_name(
        &self,
        geometry: Geometry,
        material: Material,
        name: &str,
    ) -> Node {
        self.object_from_mesh_and_info(
            Mesh { geometry, material },
            ObjectInfo {
                name: name.into(),
                ..Default::default()
            },
        )
    }
    pub fn object_from_mesh(&self, geometry: Geometry, material: Material) -> Node {
        self.object_from_mesh_and_name(geometry, material, "node")
    }
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
    fn add_viewport_events(&self) {
        let window = window();
        let perf = window.performance().unwrap();

        let renderer = self.renderer.borrow();
        let canvas = renderer.canvas();

        let a_view = self.viewport.clone();
        add_event(&canvas, "mousemove", move |e| {
            let me = e.dyn_into::<MouseEvent>().unwrap();
            let dt = perf.now();
            a_view
                .borrow_mut()
                .update_rot(me.movement_x(), me.movement_y(), dt as f32);
        });

        let b_view = self.viewport.clone();
        add_event(&canvas, "wheel", move |e| {
            let mut view = b_view.borrow_mut();
            let we = e.dyn_into::<WheelEvent>().unwrap();
            view.enable_zoom();
            view.update_zoom(we.delta_y() as i32);
            view.disable_zoom();
        });

        if let Some(button) = self.viewport.borrow().button() {
            let a_view = self.viewport.clone();
            let a_rndr = self.renderer.clone();
            add_event(canvas, "mousedown", move |e| {
                let mut view = a_view.borrow_mut();
                let renderer = a_rndr.borrow_mut();
                let me = e.dyn_into::<MouseEvent>().unwrap();
                if me.button() == button as i16 {
                    renderer.change_cursor(CursorType::Grab);
                    view.enable_rotation();
                }
                if me.button() == MouseButton::MIDDLE as i16 {
                    view.enable_zoom();
                }
            });
            let a_view = self.viewport.clone();
            let a_rndr = self.renderer.clone();
            add_event(&window, "mouseup", move |e| {
                let mut view = a_view.borrow_mut();
                let renderer = a_rndr.borrow_mut();
                let me = e.dyn_into::<MouseEvent>().unwrap();
                let pressed_btn = me.button();
                if (pressed_btn == button as i16) || (pressed_btn == MouseButton::MIDDLE as i16) {
                    renderer.change_cursor(CursorType::Pointer);
                    view.disable_rotation();
                    view.disable_zoom()
                }
            });
        }

        let a_view = self.viewport.clone();
        add_event(&window, "keydown", move |e| {
            let keycode = e.dyn_into::<KeyboardEvent>().unwrap().code();
            if keycode == "KeyR" {
                a_view.borrow_mut().reset();
            }
        });

        let a_rndr = self.renderer.clone();
        let a_view = self.viewport.clone();
        add_event(&window, "resize", move |_| {
            let mut renderer = a_rndr.borrow_mut();
            renderer.resize();
            a_view.borrow_mut().resize(renderer.aspect_ratio());
        });
    }
}
