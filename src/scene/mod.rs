mod node;
pub mod primitives;
mod storage;

use std::collections::HashMap;

#[doc(inline)]
pub use primitives::Primitive;

#[doc(inline)]
pub use node::Node;
pub use storage::Storage;

use crate::{
    dom_factory::{add_event, window, now, set_timeout, request_animation_frame},
    log, node, rc_rcell, TextureType,
    renderer::{bind_texture, CursorType, DrawMode, RenderFlags, Renderer},
    scene::primitives::create_light_node,
    Geometry, Material, Mesh, MouseButton, RcRcell, Transform, Viewport,
};
use genmesh::generators::Cube;
use nalgebra::Vector3;
use strum_macros::{Display, EnumIter, EnumString};
use wasm_bindgen::{JsCast, closure::Closure};
use wavefront_obj::{mtl, obj};
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
                name: "Scene".into(),
                ..Default::default()
            },
            false,
            false,
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
            info.render_flags.render = visible;
        }
        for child in node.owned_children() {
            self.set_visibility_only(child, visible);
        }
    }
    pub fn set_skybox(&self, dir: &str, ext: &str) {
        let mesh = Mesh::new(
            Geometry::from_genmesh(&Cube::new()),
            Material::new_cube_map(
                [
                    &format!("{}/posx.{}",dir,ext),
                    &format!("{}/negx.{}",dir,ext),
                    &format!("{}/posy.{}",dir,ext),
                    &format!("{}/negy.{}",dir,ext),
                    &format!("{}/posz.{}",dir,ext),
                    &format!("{}/negz.{}",dir,ext),
                ],
            ),
        );
        let cube = node!(&self, Some(mesh), RenderFlags::no_cull());
        self.show(&cube);
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
        is_img_obj: bool,
        setup_unique_vertices: bool,
    ) -> Node {
        let sto = storage.clone();
        let mut storage = storage.borrow_mut();
        let index = if let Some(mut mesh) = mesh {
            if setup_unique_vertices {
                mesh.setup_unique_vertices();
            }
            let vao = renderer.create_vao(&mesh);
            let urls = &mesh.material.texture_urls[..];
            if urls.len() > 0 {
                let tex_i = storage.add_texture(
                    bind_texture(renderer.context(), urls, mesh.material.tex_type, is_img_obj)
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
    pub fn from_mesh(&self, mesh: Option<Mesh>, setup_unique_vertices: bool) -> Node {
        Self::object(
            self.storage(),
            &self.renderer.borrow(),
            mesh,
            Default::default(),
            Default::default(),
            false,
            setup_unique_vertices,
        )
    }
    pub fn empty(&self, name: &str) -> Node {
        Self::object(
            self.storage(),
            &self.renderer.borrow(),
            None,
            Default::default(),
            ObjectInfo {
                name: name.to_string(),
                ..Default::default()
            },
            false,
            false,
        )
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
    pub fn load_object_from_obj_wired(
        &self,
        dir: &str,
        object: &obj::Object,
        mat_set: &Option<mtl::MtlSet>,
        img_obj_url: Option<&HashMap<String, String>>,
    ) -> Node {
        let mut buf_vertices = Vec::new();
        for vertex in &object.vertices {
            buf_vertices.push(vertex.x as f32);
            buf_vertices.push(vertex.y as f32);
            buf_vertices.push(vertex.z as f32);
        }
        let mut buf_normals = Vec::new();
        for normal in &object.normals {
            buf_normals.push(normal.x as f32);
            buf_normals.push(normal.y as f32);
            buf_normals.push(normal.z as f32);
        }
        let mut buf_tex_coords = Vec::new();
        for normal in &object.normals {
            buf_tex_coords.push(normal.x as f32);
            buf_tex_coords.push(normal.y as f32);
            buf_tex_coords.push(normal.z as f32);
        }
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();
        let mut tex_coords = Vec::new();
        for shape in &object.geometry[0].shapes {
            if let obj::Primitive::Triangle(a, b, c) = shape.primitive {
                if a.1 != None && a.2 != None {
                    for ((a, ua), na) in [a.0, b.0, c.0]
                        .iter()
                        .zip([a.1.unwrap(), b.1.unwrap(), c.1.unwrap()].iter())
                        .zip([a.2.unwrap(), b.2.unwrap(), c.2.unwrap()].iter())
                    {
                        let (e, ue, ne) = (*a * 3, *ua, *na);
                        indices.push(e as u16);
                        vertices.push(buf_vertices[e]);
                        vertices.push(buf_vertices[e + 1]);
                        vertices.push(buf_vertices[e + 2]);
                        normals.push(buf_vertices[e]);
                        normals.push(buf_vertices[e + 1]);
                        normals.push(buf_vertices[e + 2]);
                        let u = object.tex_vertices[ue].u as f32;
                        let v = -object.tex_vertices[ue].v as f32;
                        tex_coords.push(u);
                        tex_coords.push(v);
                    }
                } else {
                    log!("obj file doesn't have normal or uv indices. Only vertices are loaded");
                }
            }
        }
        let geometry = Geometry {
            vertices,
            indices,
            normals,
        };
        let material = Self::load_material(dir, object, mat_set, tex_coords, img_obj_url);
        let node = self.from_mesh(Some(Mesh { geometry, material }), false);
        let mut info = node.info();
        info.draw_mode = DrawMode::Arrays;
        info.name = object.name.clone();
        node.set_info(info);
        node
    }
    pub fn load_material(
        dir: &str,
        object: &obj::Object,
        mat_set: &Option<mtl::MtlSet>,
        tex_coords: Vec<f32>,
        img_obj_url: Option<&HashMap<String, String>>,
    ) -> Material {
        let mut material = Material::new_color(1., 1., 1., 1.);
        if let Some(material_name) = &object.geometry[0].material_name {
            if let Some(mat_set) = mat_set {
                for each in &mat_set.materials {
                    if &each.name == material_name {
                        let c = each.color_diffuse;
                        material = Material::new_color(
                            c.r as f32,
                            c.g as f32,
                            c.b as f32,
                            each.alpha as f32,
                        );
                        if let Some(name) = &each.uv_map {
                            let url = if let Some(urls) = img_obj_url {
                                if let Some(url) = urls.get(name) {
                                    Some(url.to_string())
                                } else {
                                    log!("The file with name " name.to_string() " was not uploaded. Resorting to color");
                                    None
                                }
                            } else {
                                if dir != "" {
                                    Some(format!("{}/{}", dir, name))
                                } else {
                                    log!("Invalid texture path! Won't load any texture for " name.to_string() ".");
                                    None
                                }
                            };
                            if let Some(url) = url {
                                material = material.tex_type(TextureType::Tex2d).tex_coords(tex_coords).texture(&url);
                            }
                        }
                        if object.geometry[0].shapes[0].smoothing_groups.is_empty() {
                            material = material.flat();
                        }
                        return material;
                    }
                }
            }
        }
        material
    }
    pub fn load_object_from_obj(
        &self,
        dir: &str,
        object: &obj::Object,
        mat_set: &Option<mtl::MtlSet>,
        img_obj_url: Option<&HashMap<String, String>>,
    ) -> Node {
        let mut vertices = Vec::new();
        for vertex in &object.vertices {
            vertices.push(vertex.x as f32);
            vertices.push(vertex.y as f32);
            vertices.push(vertex.z as f32);
        }
        let mut indices = Vec::new();
        let mut normals: Vec<f32> = vec![0.; vertices.len()];
        let mut tex_coords: Vec<f32> = vec![-1.; vertices.len() / 3 * 2];
        let push_normals = |normals: &mut Vec<f32>, e: usize, ne: usize| {
            let current_normal = Vector3::new(normals[e], normals[e + 1], normals[e + 2]);
            let new_normal = Vector3::new(
                object.normals[ne].x as f32,
                object.normals[ne].y as f32,
                object.normals[ne].z as f32,
            )
            .normalize();
            let avg_normal = if current_normal.magnitude() == 1. {
                ((new_normal + current_normal) * 0.5).normalize()
            } else {
                new_normal
            };
            normals[e] = avg_normal.x;
            normals[e + 1] = avg_normal.y;
            normals[e + 2] = avg_normal.z;
        };
        for shape in &object.geometry[0].shapes {
            if let obj::Primitive::Triangle(a, b, c) = shape.primitive {
                if a.1 != None && a.2 != None {
                    for ((a, ua), na) in [a.0, b.0, c.0]
                        .iter()
                        .zip([a.1.unwrap(), b.1.unwrap(), c.1.unwrap()].iter())
                        .zip([a.2.unwrap(), b.2.unwrap(), c.2.unwrap()].iter())
                    {
                        let (e, ue, ne) = (*a * 3, *ua, *na);
                        push_normals(&mut normals, e, ne);
                        let u = object.tex_vertices[ue].u as f32;
                        let v = -object.tex_vertices[ue].v as f32;

                        let current_tex = tex_coords[*a * 2];
                        let duplicate_v = current_tex != -1. && current_tex != u;
                        let current_tex = tex_coords[*a * 2 + 1];
                        let duplicate_u = current_tex != -1. && current_tex != v;
                        if duplicate_u || duplicate_v {
                            indices.push((vertices.len() / 3) as u16);
                            vertices.push(vertices[e]);
                            vertices.push(vertices[e + 1]);
                            vertices.push(vertices[e + 2]);
                            normals.push(normals[e]);
                            normals.push(normals[e + 1]);
                            normals.push(normals[e + 2]);
                            tex_coords.push(u);
                            tex_coords.push(v);
                        } else {
                            indices.push(*a as u16);
                            tex_coords[*a * 2] = u;
                            tex_coords[*a * 2 + 1] = v;
                        }
                    }
                } else {
                    log!("obj file doesn't have normal or uv indices. Only vertices are loaded");
                }
            }
        }
        let geometry = Geometry {
            vertices,
            indices,
            normals,
        };
        let material = Self::load_material(dir, object, mat_set, tex_coords, img_obj_url);
        node!(
            &self,
            Some(Mesh { geometry, material }),
            object.name.clone()
        )
    }
    pub fn object_from_obj(
        &self,
        dir: &str,
        obj_src: &str,
        mtl_src: Option<&str>,
        img_obj_url: Option<&HashMap<String, String>>,
        wire_overlay: bool,
    ) -> Node {
        let obj_set = obj::parse(obj_src).unwrap();
        assert!(!obj_set.objects.is_empty(), "No objects in the obj file");
        let mat_set = if let Some(src) = mtl_src {
            Some(mtl::parse(src).unwrap())
        } else {
            None
        };
        if wire_overlay {
            let mut root =
                self.load_object_from_obj_wired(dir, &obj_set.objects[0], &mat_set, img_obj_url);
            for object in obj_set.objects.iter().skip(1) {
                root.add(rc_rcell(self.load_object_from_obj_wired(
                    dir,
                    &object,
                    &mat_set,
                    img_obj_url,
                )));
            }
            root
        } else {
            let mut root =
                self.load_object_from_obj(dir, &obj_set.objects[0], &mat_set, img_obj_url);
            for object in obj_set.objects.iter().skip(1) {
                root.add(rc_rcell(self.load_object_from_obj(
                    dir,
                    &object,
                    &mat_set,
                    img_obj_url,
                )));
            }
            root
        }
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
            false,
            false,
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
