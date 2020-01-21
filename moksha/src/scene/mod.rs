mod object;
pub mod primitives;
mod storage;
use std::collections::HashMap;

#[doc(inline)]
pub use primitives::Primitive;

#[doc(inline)]
pub use object::Object;
pub use storage::{Storage, Id};
use crate::{
    dom_factory::{add_event, window, now, set_timeout, request_animation_frame,
    loop_animation_frame},
    log, object, rc_rcell, TextureType,
    renderer::{bind_texture, DrawMode, RenderFlags, Renderer},
    Events,
    events::{ViewportEvent,CanvasEvent},
    scene::primitives::create_light_node,
    Geometry, Material, Mesh, MouseButton, RcRcell, Transform, Viewport,
};
use genmesh::generators::Cube;
use nalgebra::Vector3;
use strum_macros::{Display, EnumIter, EnumString};
use wasm_bindgen::{JsCast, closure::Closure};
use wavefront_obj::{mtl, obj};
use web_sys::{MouseEvent, WheelEvent, HtmlCanvasElement};

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
    object: RcRcell<Object>,
}

impl Light {
    pub fn object(&self) -> RcRcell<Object> {
        self.object.clone()
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
            name: "object".into(),
            draw_mode: DrawMode::Triangle,
            render_flags: Default::default(),
        }
    }
}

/// A Scene tree that facilitates creation of varieties of Nodes. Scene creates Storage that is
/// then shared by all nodes.
pub struct Scene {
    root: RcRcell<Object>,
    renderer: RcRcell<Renderer>,
    viewport: RcRcell<Viewport>,
    events: RcRcell<Events>,
}

impl Scene {
    pub fn new(renderer: RcRcell<Renderer>, viewport: RcRcell<Viewport>, events: RcRcell<Events>) -> Self {
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
        Self::add_events(events.clone(), &renderer.borrow().canvas(), viewport.borrow().button());
        Self {
            root,
            renderer,
            viewport,
            events,
        }
    }
    pub fn root(&self) -> RcRcell<Object> {
        self.root.clone()
    }
    pub fn view(&self) -> RcRcell<Viewport> {
        self.viewport.clone()
    }
    pub fn renderer(&self) -> RcRcell<Renderer> {
        self.renderer.clone()
    }
    pub fn turn_lights_visiblity(&self, object: &Object, visible: bool) {
        let s = self.storage();
        let mut storage = s.borrow_mut();
        for i in 0..storage.lights().len() {
            if storage.light(i).node_id == object.index() {
                storage.mut_light_info(i).light = visible;
                break;
            }
        }
    }
    pub fn turn_lights_on(&self, object: &Object) {
        self.turn_lights_visiblity(object, true);
    }
    pub fn turn_lights_off(&self, object: &Object) {
        self.turn_lights_visiblity(object, false);
    }
    pub fn show(&self, object: &Object) {
        {
            let s = self.storage();
            let mut storage = s.borrow_mut();
            let mut info = storage.mut_info(object.index());
            info.render_flags.render = true;
        }
        for child in object.children() {
            self.show(&child.borrow());
        }
        for child in object.owned_children() {
            self.show(child);
        }
    }
    pub fn set_visibility_only(&self, object: &Object, visible: bool) {
        {
            let s = self.storage();
            let mut storage = s.borrow_mut();
            let mut info = storage.mut_info(object.index());
            info.render_flags.render = visible;
        }
        for child in object.owned_children() {
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
        let cube = object!(&self, Some(mesh), "Skybox", RenderFlags::no_cull());
        self.show(&cube);
    }
    pub fn show_only(&self, object: &Object) {
        self.set_visibility_only(object, true);
    }
    pub fn hide_only(&self, object: &Object) {
        self.set_visibility_only(object, false);
    }
    pub fn add(&self, object: RcRcell<Object>) {
        self.show(&object.borrow());
        self.root.borrow_mut().add(object);
    }
    pub fn add_light(&self, light: &Light) {
        self.add(light.object());
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
    ) -> Object {
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
        Object::new(index, sto)
    }
    pub fn from_mesh(&self, mesh: Option<Mesh>, setup_unique_vertices: bool) -> Object {
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
    pub fn empty(&self, name: &str) -> Object {
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
        let object = rc_rcell(create_light_node(&self, light_type, color));
        let light_id = self.storage().borrow_mut().add_light(LightInfo {
            light_type,
            intensity,
            color,
            node_id: object.borrow().index(),
            light: false,
        });
        Light { light_id, object }
    }
    pub fn load_object_from_obj_wired(
        &self,
        dir: &str,
        obj_object: &obj::Object,
        mat_set: &Option<mtl::MtlSet>,
        img_obj_url: Option<&HashMap<String, String>>,
    ) -> Object {
        let mut buf_vertices = Vec::new();
        for vertex in &obj_object.vertices {
            buf_vertices.push(vertex.x);
            buf_vertices.push(vertex.y);
            buf_vertices.push(vertex.z);
        }
        let mut buf_normals = Vec::new();
        for normal in &obj_object.normals {
            buf_normals.push(normal.x);
            buf_normals.push(normal.y);
            buf_normals.push(normal.z);
        }
        let mut buf_tex_coords = Vec::new();
        for normal in &obj_object.normals {
            buf_tex_coords.push(normal.x);
            buf_tex_coords.push(normal.y);
            buf_tex_coords.push(normal.z);
        }
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();
        let mut tex_coords = Vec::new();
        for shape in &obj_object.geometry[0].shapes {
            if let obj::Primitive::Triangle(a, b, c) = shape.primitive {
                if a.1 != None && a.2 != None {
                    for ((a, ua), na) in [a.0, b.0, c.0]
                        .iter()
                        .zip([a.1.unwrap(), b.1.unwrap(), c.1.unwrap()].iter())
                        .zip([a.2.unwrap(), b.2.unwrap(), c.2.unwrap()].iter())
                    {
                        let (e, ue, ne) = (*a * 3, *ua, *na);
                        indices.push(e as u16);
                        vertices.push(buf_vertices[e] as f32);
                        vertices.push(buf_vertices[e + 1] as f32);
                        vertices.push(buf_vertices[e + 2] as f32);
                        normals.push(buf_vertices[e] as f32);
                        normals.push(buf_vertices[e + 1] as f32);
                        normals.push(buf_vertices[e + 2] as f32);
                        let u = obj_object.tex_vertices[ue].u as f64;
                        let v = -obj_object.tex_vertices[ue].v as f64;
                        tex_coords.push(u as f32);
                        tex_coords.push(v as f32);
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
        let material = Self::load_material(dir, obj_object, mat_set, tex_coords, img_obj_url);
        let object = self.from_mesh(Some(Mesh { geometry, material }), false);
        let mut info = object.info();
        info.draw_mode = DrawMode::Arrays;
        info.name = obj_object.name.clone();
        object.set_info(info);
        object
    }
    pub fn load_material(
        dir: &str,
        obj_object: &obj::Object,
        mat_set: &Option<mtl::MtlSet>,
        tex_coords: Vec<f32>,
        img_obj_url: Option<&HashMap<String, String>>,
    ) -> Material {
        let mut material = Material::new_color(1., 1., 1., 1.);
        if obj_object.geometry[0].shapes[0].smoothing_groups.is_empty() {
            log!("No smooting group found. Mesh will be rendered flat.");
            material = material.flat();
        }
        if let Some(material_name) = &obj_object.geometry[0].material_name {
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
        obj_object: &obj::Object,
        mat_set: &Option<mtl::MtlSet>,
        img_obj_url: Option<&HashMap<String, String>>,
    ) -> Object {
        let mut vertices = Vec::new();
        for vertex in &obj_object.vertices {
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
                obj_object.normals[ne].x as f32,
                obj_object.normals[ne].y as f32,
                obj_object.normals[ne].z as f32,
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
        for shape in &obj_object.geometry[0].shapes {
            if let obj::Primitive::Triangle(a, b, c) = shape.primitive {
                if a.1 != None && a.2 != None {
                    for ((a, ua), na) in [a.0, b.0, c.0]
                        .iter()
                        .zip([a.1.unwrap(), b.1.unwrap(), c.1.unwrap()].iter())
                        .zip([a.2.unwrap(), b.2.unwrap(), c.2.unwrap()].iter())
                    {
                        let (e, ue, ne) = (*a * 3, *ua, *na);
                        push_normals(&mut normals, e, ne);
                        let u = obj_object.tex_vertices[ue].u as f32;
                        let v = -obj_object.tex_vertices[ue].v as f32;

                        let current_tex = tex_coords[*a * 2];
                        let duplicate_v = current_tex != -1. && current_tex != u;
                        let current_tex = tex_coords[*a * 2 + 1];
                        let duplicate_u = current_tex != -1. && current_tex != v;
                        if duplicate_u || duplicate_v {
                            indices.push((vertices.len() / 3) as u16);
                            vertices.push(vertices[e] as f32);
                            vertices.push(vertices[e + 1] as f32);
                            vertices.push(vertices[e + 2] as f32);
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
        let material = Self::load_material(dir, obj_object, mat_set, tex_coords, img_obj_url);
        object!(
            &self,
            Some(Mesh { geometry, material }),
            obj_object.name.clone()
        )
    }
    pub fn object_from_obj(
        &self,
        dir: &str,
        obj_src: &str,
        mtl_src: Option<&str>,
        img_obj_url: Option<&HashMap<String, String>>,
        wire_overlay: bool,
    ) -> Object {
        let obj_set = obj::parse(obj_src).unwrap();
        assert!(!obj_set.objects.is_empty(), "No objects in the obj file");
        if obj_set.objects.len() > 1 {
            log!("Please note that the obj file has multiple objects. Hence they'll have a single origin which might affect transformations and object outlining.");
        }
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
    pub fn find_node_recursive(object: RcRcell<Object>, name: &str) -> Option<RcRcell<Object>> {
        if object.borrow().info().name.as_str() == name {
            Some(object)
        } else {
            for each in object.borrow().children() {
                if let Some(n) = Self::find_node_recursive(each.clone(), name) {
                    return Some(n);
                }
            }
            None
        }
    }
    pub fn find_node_w_name(&self, name: &str) -> Option<RcRcell<Object>> {
        Self::find_node_recursive(self.root(), name)
    }
    pub fn duplicate_node(&self, object: &Object) -> Object {
        let transform = object.transform();
        let info = object.info();
        let mesh = object.mesh();
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
    fn add_events(events: RcRcell<Events>, canvas: &HtmlCanvasElement, button: Option<MouseButton>) {
        let ev = events.clone();
        add_event(&canvas, "mousemove", move |e| {
            let mut events = ev.borrow_mut();
            let me = e.dyn_into::<MouseEvent>().unwrap();
            events.viewport = ViewportEvent::Rotate(me.movement_x(), me.movement_y());
        });
        let ev = events.clone();
        add_event(&canvas, "wheel", move |e| {
            let mut events = ev.borrow_mut();
            let we = e.dyn_into::<WheelEvent>().unwrap();
            events.viewport = ViewportEvent::Zoom(we.delta_y());
        });
        let window = window();
        if let Some(button) = button {
            let ev = events.clone();
            add_event(canvas, "mousedown", move |e| {
                let mut events = ev.borrow_mut();
                let me = e.dyn_into::<MouseEvent>().unwrap();
                if me.button() == button as i16 {
                    events.canvas = CanvasEvent::Grab
                }
                if me.button() == MouseButton::MIDDLE as i16 {
                    events.canvas = CanvasEvent::Zoom
                }
            });
            let ev = events.clone();
            add_event(&window, "mouseup", move |e| {
                let mut events = ev.borrow_mut();
                let me = e.dyn_into::<MouseEvent>().unwrap();
                let pressed_btn = me.button();
                if (pressed_btn == button as i16) || (pressed_btn == MouseButton::MIDDLE as i16) {
                    events.canvas = CanvasEvent::Point
                }
            });
        }
        let ev = events.clone();
        add_event(&window, "resize", move |_| {
            let mut events = ev.borrow_mut();
            events.canvas = CanvasEvent::Resize;
        });
    }
    pub fn update<F>(&self, closure: F)
    where
        F: 'static + Fn(&Events, f64) {
        let then = rc_rcell(now());
        let storage = self.storage();
        let events = self.events.clone();
        let view = self.viewport.clone();
        let rndr = self.renderer.clone();
        let scene = self.clone();
        loop_animation_frame(move || {
            let mut then = then.borrow_mut();
            let mut events = events.borrow_mut();
            let mut rndr = rndr.borrow_mut();
            let mut view = view.borrow_mut();
            let dt = now() - *then;
            view.update(&events, dt);
            Self::update_canvas(&events, &mut rndr, &mut view);
            closure(&events, dt);
            rndr.render(&storage.borrow(), &view);
            events.clear();
            *then = now(); 
        }, Some(60.));
    }
    fn update_canvas(events: &Events, renderer: &mut Renderer, viewport: &mut Viewport) {
        let canvas_style = renderer.canvas().style();
        match events.canvas {
            CanvasEvent::Point => {
                canvas_style
                    .set_property("cursor", "var(--cursor-auto)")
                    .unwrap();
            }
            CanvasEvent::Grab => {
                canvas_style
                    .set_property("cursor", "var(--cursor-grab)")
                    .unwrap();
            }
            CanvasEvent::Zoom => {
                canvas_style
                    .set_property("cursor", "var(--cursor-zoom)")
                    .unwrap();
            }
            CanvasEvent::Resize => {
                renderer.resize();
                viewport.resize(renderer.aspect_ratio());
            }
        }
    }
}

mod node;
use node::Obj;
use moksha_derive::Obj;

#[derive(Obj)]
pub struct LightO {
    obj_id: Id,
    storage: RcRcell<Storage>,
}
