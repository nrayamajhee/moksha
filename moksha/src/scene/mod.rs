mod light;
mod object;
pub mod primitives;
mod storage;

#[doc(inline)]
pub use light::{Light, LightInfo, LightState, LightType};
#[doc(inline)]
pub use object::{prelude, Object, ObjectInfo, SceneObject};
#[doc(inline)]
pub use primitives::Primitive;
#[doc(inline)]
pub use storage::{Id, Storage};

use crate::{
    dom_factory::{add_event, loop_animation_frame, now, window},
    events::{CanvasEvent, ViewportEvent},
    log, object, rc_rcell,
    renderer::{bind_texture, DrawMode, Renderer},
    Color, Events, Geometry, Material, Mesh, MouseButton, RcRcell, Transform, Viewport,
};
use object::prelude::*;
use primitives::create_light;

use std::collections::HashMap;
use genmesh::generators::Cube;
use wasm_bindgen::JsCast;
use wavefront_obj::{mtl, obj};
use web_sys::{HtmlCanvasElement, MouseEvent, WheelEvent};

/// A Scene tree that facilitates creation of varieties of Nodes. Scene creates Storage that is
/// then shared by all nodes.
pub struct Scene {
    root: SceneObject,
    renderer: RcRcell<Renderer>,
    viewport: RcRcell<Viewport>,
    events: RcRcell<Events>,
}

impl Scene {
    fn object(
        storage: RcRcell<Storage>,
        renderer: &Renderer,
        mesh: Option<Mesh>,
        transform: Transform,
        info: ObjectInfo,
        is_img_object: bool,
        setup_unique_vertices: bool,
    ) -> SceneObject {
        let sto = storage.clone();
        let mut storage = storage.borrow_mut();
        let obj_id = if let Some(mut mesh) = mesh {
            if setup_unique_vertices {
                mesh.setup_unique_vertices();
            }
            let vao = renderer.create_vao(&mesh);
            let urls = &mesh.material.texture_urls[..];
            if urls.len() > 0 {
                let tex_i = storage.add_texture(
                    bind_texture(
                        renderer.context(),
                        urls,
                        mesh.material.tex_type,
                        is_img_object,
                    )
                    .expect("Couldn't bind albedo texture"),
                );
                mesh.material.texture_indices.push(tex_i);
            }
            storage.add(Some(mesh), Some(vao), transform, info)
        } else {
            storage.add(None, None, transform, info)
        };
        SceneObject::new(
            sto,
            obj_id
        )
    }
    pub fn new(
        renderer: RcRcell<Renderer>,
        viewport: RcRcell<Viewport>,
        events: RcRcell<Events>,
    ) -> Self {
        let storage = rc_rcell(Default::default());
        let root = Self::object(
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
        );
        Self::add_events(
            events.clone(),
            &renderer.borrow().canvas(),
            viewport.borrow().button(),
        );
        Self {
            root,
            renderer,
            viewport,
            events,
        }
    }
    pub fn root(&self) -> &SceneObject {
        &self.root
    }
    pub fn view(&self) -> RcRcell<Viewport> {
        self.viewport.clone()
    }
    pub fn renderer(&self) -> RcRcell<Renderer> {
        self.renderer.clone()
    }
    pub fn from_mesh(&self, mesh: Option<Mesh>, setup_unique_vertices: bool) -> SceneObject {
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
    pub fn empty(&self, name: &str) -> SceneObject {
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
    pub fn show(&self, object: &SceneObject) {
        let mut info = object.info();
        info.render_flags.render = true;
        object.set_info(info);
        for child in object.children() {
            self.show(&child);
        }
    }
    pub fn add(&self, object: &SceneObject) {
        self.show(object.into());
        //self.root.add(object.into());
    }
    pub fn add_light(&self, light: &Light) {
        self.add(&light.into());
        let mut info = light.light_info();
        info.state = LightState::On;
        light.set_light_info(info);
    }
    pub fn light(&self, light_type: LightType, color: Color, intensity: f32) -> Light {
        let object = create_light(&self, light_type, color);
        let obj_id = object.obj_id();
        let light_id = self.storage().borrow_mut().add_light(LightInfo {
            light_type,
            intensity,
            obj_id,
            color,
            state: LightState::Off,
        });
        Light::new(
            self.storage(),
            light_id,
            obj_id,
        )
    }
    //pub fn turn_lights(&self, object: &Object, light_state: LightState) {
    //let s = self.storage();
    //let mut storage = s.borrow_mut();
    //for i in 0..storage.lights().len() {
    //if storage.light(i).node_id == object.index() {
    //storage.mut_light_info(i).light = light_state.into();
    //break;
    //}
    //}
    //}
    //pub fn turn_lights_on(&self, object: &Object) {
    //self.turn_lights_visiblity(object, true);
    //}
    //pub fn turn_lights_off(&self, object: &Object) {
    //self.turn_lights_visiblity(object, false);
    //}
    //pub fn set_visibility_only(&self, object: &Object, visible: bool) {
    //{
    //let s = self.storage();
    //let mut storage = s.borrow_mut();
    //let mut info = storage.mut_info(object.index());
    //info.render_flags.render = visible;
    //}
    //for child in object.owned_children() {
    //self.set_visibility_only(child, visible);
    //}
    //}
    //pub fn show_only(&self, object: &Object) {
    //self.set_visibility_only(object, true);
    //}
    //pub fn hide_only(&self, object: &Object) {
    //self.set_visibility_only(object, false);
    //}
    pub fn set_skybox(&self, dir: &str, ext: &str) {
        let mesh = Mesh::new(
            Geometry::from_genmesh(&Cube::new()),
            Material::new_cube_map([
                &format!("{}/posx.{}", dir, ext),
                &format!("{}/negx.{}", dir, ext),
                &format!("{}/posy.{}", dir, ext),
                &format!("{}/negy.{}", dir, ext),
                &format!("{}/posz.{}", dir, ext),
                &format!("{}/negz.{}", dir, ext),
            ]),
        );
        let cube = object!(&self, Some(mesh), "Skybox", RenderFlags::no_cull());
        self.show(&cube);
    }
    pub fn load_object_from_obj_wired(
        &self,
        dir: &str,
        obj_object: &obj::Object,
        mat_set: &Option<mtl::MtlSet>,
        img_obj_url: Option<&HashMap<String, String>>,
    ) -> SceneObject {
        let geometry = Geometry::from_obj(&obj_object);
        let material = Material::from_obj(dir, obj_object, mat_set, img_obj_url);
        let object = self.from_mesh(Some(Mesh { geometry, material }), false);
        let mut info = object.info();
        info.draw_mode = DrawMode::Arrays;
        info.name = obj_object.name.clone();
        object.set_info(info);
        object
    }
    pub fn load_object_from_obj(
        &self,
        dir: &str,
        obj_object: &obj::Object,
        mat_set: &Option<mtl::MtlSet>,
        img_obj_url: Option<&HashMap<String, String>>,
    ) -> SceneObject {
        let geometry = Geometry::from_obj(&obj_object);
        let material = Material::from_obj(dir, obj_object, mat_set, img_obj_url);
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
    ) -> SceneObject {
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
                root.add(&self.load_object_from_obj_wired(dir, &object, &mat_set, img_obj_url));
            }
            root
        } else {
            let mut root =
                self.load_object_from_obj(dir, &obj_set.objects[0], &mat_set, img_obj_url);
            for object in obj_set.objects.iter().skip(1) {
                root.add(&self.load_object_from_obj(dir, &object, &mat_set, img_obj_url));
            }
            root
        }
    }
    pub fn storage(&self) -> RcRcell<Storage> {
        self.root.storage()
    }
    pub fn find_object(&self, name: &str) -> Option<SceneObject> {
        if let Some((child_id, _)) = self.root.find_child(name) {
            Some(SceneObject::new( 
                self.storage(),
                child_id,
            ))
        } else {
            None
        }
    }
    pub fn duplicate_node(&self, object: &SceneObject) -> SceneObject {
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
    fn add_events(
        events: RcRcell<Events>,
        canvas: &HtmlCanvasElement,
        button: Option<MouseButton>,
    ) {
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
        F: 'static + Fn(&Events, f64),
    {
        let then = rc_rcell(now());
        let storage = self.storage();
        let events = self.events.clone();
        let view = self.viewport.clone();
        let rndr = self.renderer.clone();
        let scene = self.clone();
        loop_animation_frame(
            move || {
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
            },
            Some(60.),
        );
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
