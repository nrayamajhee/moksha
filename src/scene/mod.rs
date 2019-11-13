mod node;
mod storage;

pub mod primitives;

#[doc(inline)]
pub use primitives::Primitive;

#[doc(inline)]
pub use node::Node;
pub use storage::Storage;

use crate::{
    dom_factory::{add_event, window},
    rc_rcell,
    renderer::{bind_texture, CursorType, DrawMode, RenderFlags, Renderer},
    scene::primitives::create_light_node,
    Geometry, Material, Mesh, MouseButton, RcRcell, Transform, Viewport,
};
use nalgebra::Vector3;
use strum_macros::{Display, EnumIter, EnumString};
use wasm_bindgen::JsCast;
use wavefront_obj::obj;
use web_sys::{MouseEvent, WheelEvent};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Display, EnumIter, EnumString)]
pub enum LightType {
    Ambient,
    Point,
    Directional,
    Spot,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LightInfo {
    pub light_type: LightType,
    pub intensity: f32,
    pub color: [f32; 3],
    pub node_id: usize,
    pub light: bool,
}

pub struct Light {
    light_id: usize,
    node: RcRcell<Node>,
}

impl Light {
    pub fn node(&self) -> RcRcell<Node> {
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
    pub render_flags: RenderFlags,
}

impl Default for ObjectInfo {
    fn default() -> Self {
        Self {
            name: "node".into(),
            draw_mode: DrawMode::Triangle,
            render_flags: Default::default(),
        }
    }
}

/// A Scene tree that facilitates creation of varieties of Nodes. Scene creates Storage that is
/// then shared by all nodes.
pub struct Scene {
    root: RcRcell<Node>,
    renderer: RcRcell<Renderer>,
    viewport: RcRcell<Viewport>,
}

impl Scene {
    pub fn new(renderer: RcRcell<Renderer>, viewport: RcRcell<Viewport>) -> Self {
        let storage = rc_rcell(Default::default());
        let root = rc_rcell(Self::object(
            storage,
            &renderer.borrow(),
            None,
            Default::default(),
            ObjectInfo {
                name: "root".into(),
                ..Default::default()
            },
        ));
        let scene = Self {
            root,
            renderer,
            viewport,
        };
        scene.add_viewport_events();
        scene
    }
    pub fn root(&self) -> RcRcell<Node> {
        self.root.clone()
    }
    pub fn view(&self) -> RcRcell<Viewport> {
        self.viewport.clone()
    }
    pub fn renderer(&self) -> RcRcell<Renderer> {
        self.renderer.clone()
    }
    pub fn turn_lights_visiblity(&self, node: &Node, visible: bool) {
        let s = self.storage();
        let mut storage = s.borrow_mut();
        for i in 0..storage.lights().len() {
            if storage.light(i).node_id == node.index() {
                storage.mut_light_info(i).light = visible;
                break;
            }
        }
    }
    pub fn turn_lights_on(&self, node: &Node) {
        self.turn_lights_visiblity(node, true);
    }
    pub fn turn_lights_off(&self, node: &Node) {
        self.turn_lights_visiblity(node, false);
    }
    pub fn show(&self, node: &Node) {
        {
            let s = self.storage();
            let mut storage = s.borrow_mut();
            let mut info = storage.mut_info(node.index());
            info.render_flags.render = true;
        }
        for child in node.children() {
            self.show(&child.borrow());
        }
        for child in node.owned_children() {
            self.show(child);
        }
    }
    pub fn set_visibility_only(&self, node: &Node, visible: bool) {
        {
            let s = self.storage();
            let mut storage = s.borrow_mut();
            let mut info = storage.mut_info(node.index());
            info.render_flags.render = true;
        }
        for child in node.owned_children() {
            self.set_visibility_only(child, visible);
        }
    }
    pub fn show_only(&self, node: &Node) {
        self.set_visibility_only(node, true);
    }
    pub fn hide_only(&self, node: &Node) {
        self.set_visibility_only(node, false);
    }
    pub fn add(&self, node: RcRcell<Node>) {
        self.show(&node.borrow());
        self.root.borrow_mut().add(node);
    }
    pub fn add_light(&self, light: &Light) {
        self.add(light.node());
        let s = self.storage();
        let mut storage = s.borrow_mut();
        let mut info = storage.mut_light_info(light.index());
        info.light = true;
    }
    fn object(
        storage: RcRcell<Storage>,
        renderer: &Renderer,
        mesh: Option<Mesh>,
        transform: Transform,
        info: ObjectInfo,
    ) -> Node {
        let sto = storage.clone();
        let mut storage = storage.borrow_mut();
        let index = if let Some(mut mesh) = mesh {
            let vao = renderer.create_vao(&mesh);
            let urls = &mesh.material.texture_urls;
            if urls.len() > 0 {
                let tex_i = storage.add_texture(
                    bind_texture(renderer.context(), &urls[0])
                        .expect("Couldn't bind albedo texture"),
                );
                mesh.material.texture_indices.push(tex_i);
            }
            storage.add(Some(mesh), Some(vao), transform, info)
        } else {
            storage.add(None, None, transform, info)
        };
        Node::new(index, sto)
    }
    pub fn light(&self, light_type: LightType, color: [f32; 3], intensity: f32) -> Light {
        let node = rc_rcell(create_light_node(&self, light_type, color));
        let light_id = self.storage().borrow_mut().add_light(LightInfo {
            light_type,
            intensity,
            color,
            node_id: node.borrow().index(),
            light: false,
        });
        Light { light_id, node }
    }
    pub fn from_mesh(&self, mesh: Option<Mesh>) -> Node {
        Self::object(
            self.storage(),
            &self.renderer.borrow(),
            mesh,
            Default::default(),
            Default::default(),
        )
    }
    pub fn empty(&self) -> Node {
        Self::object(
            self.storage(),
            &self.renderer.borrow(),
            None,
            Default::default(),
            Default::default(),
        )
    }
    pub fn object_from_obj(&self, src: &str, albedo_src: &str) -> Node {
        let obj_sets = obj::parse(src).unwrap();
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut buf_normals = Vec::new();
        let mut tex_coords = Vec::new();
        let object = &obj_sets.objects[0];
        let name = object.name.clone();
        for vertex in &object.vertices {
            vertices.push(vertex.x as f32);
            vertices.push(vertex.y as f32);
            vertices.push(vertex.z as f32);
        }
        let mut normals: Vec<f32> = vec![0.; vertices.len()];
        for normal in &object.normals {
            buf_normals.push(normal.x as f32);
            buf_normals.push(normal.x as f32);
            buf_normals.push(normal.x as f32);
        }
        for shape in &object.geometry[0].shapes {
            if let obj::Primitive::Triangle(x, y, z) = shape.primitive {
                indices.push(x.0 as u16);
                indices.push(y.0 as u16);
                indices.push(z.0 as u16);
                if x.2 != None {
                    let current_normal = Vector3::new(
                        buf_normals[x.2.unwrap()],
                        buf_normals[y.2.unwrap()],
                        buf_normals[z.2.unwrap()],
                    );
                    let new_normal = Vector3::new(
                        normals[x.2.unwrap()],
                        normals[y.2.unwrap()],
                        normals[z.2.unwrap()],
                    );
                    let cross = current_normal.cross(&new_normal);
                    normals.push(cross.x as f32);
                    normals.push(cross.y as f32);
                    normals.push(cross.z as f32);
                } else {
                    log!("obj file doesn't have normal indices");
                }
            }
        }
        for tex_coord in &object.tex_vertices {
            tex_coords.push(tex_coord.u as f32);
            tex_coords.push(tex_coord.v as f32);
        }
        let geometry = Geometry {
            vertices,
            indices,
            normals,
        };
        //let mat = Material::new_texture(albedo_src, tex_coords).unwrap();
        let material = Material::new_color(1., 1., 1., 1.);
        //let mat = Material::new_wire(1.,1.,1.,1.);
        node!(&self, Some(Mesh { geometry, material }), name)
    }
    pub fn storage(&self) -> RcRcell<Storage> {
        self.root.borrow().storage()
    }
    pub fn find_node_recursive(node: RcRcell<Node>, name: &str) -> Option<RcRcell<Node>> {
        if node.borrow().info().name.as_str() == name {
            Some(node)
        } else {
            for each in node.borrow().children() {
                if let Some(n) = Self::find_node_recursive(each.clone(), name) {
                    return Some(n);
                }
            }
            None
        }
    }
    pub fn find_node_w_name(&self, name: &str) -> Option<RcRcell<Node>> {
        Self::find_node_recursive(self.root(), name)
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

        let a_rndr = self.renderer.clone();
        let a_view = self.viewport.clone();
        add_event(&window, "resize", move |_| {
            let mut renderer = a_rndr.borrow_mut();
            renderer.resize();
            a_view.borrow_mut().resize(renderer.aspect_ratio());
        });
    }
}
