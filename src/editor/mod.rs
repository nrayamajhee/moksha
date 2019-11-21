mod console;
mod gizmo;
mod scene_tree;
mod toolbar;
use crate::{
    dom_factory::{add_event, get_el, window},
    mesh::{Geometry, Material},
    node, rc_rcell,
    scene::{
        primitives::{create_origin, create_transform_gizmo, ArrowTip},
        Node, Scene,
    },
    Mesh, RcRcell, Viewport,
};
pub use console::{console_setup, ConsoleConfig};
use genmesh::generators::Plane;
pub use gizmo::{CollisionConstraint, Gizmo};
use nalgebra::{UnitQuaternion, Point3};
use ncollide3d::query::Ray;
use std::f32::consts::PI;
use std::rc::Rc;
use toolbar::handle_persp_toggle;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, KeyboardEvent, MouseEvent};

/// The main GUI editor that faciliates buttons to manipulate the scene, displays log in a separate
/// window, and displays the scene tree.
#[derive(Clone)]
pub struct Editor {
    scene: Rc<Scene>,
    gizmo: RcRcell<Gizmo>,
    active_node: RcRcell<Option<RcRcell<Node>>>,
    spawn_origin: RcRcell<Node>,
}

pub enum NodeRef<'a> {
    Mutable(RcRcell<Node>),
    Owned(&'a Node),
}

impl Editor {
    pub fn new(scene: Rc<Scene>) -> Self {
        let grid = node!(
            &scene,
            Some(Mesh::new(
                Geometry::from_genmesh_no_normals(&Plane::subdivide(100, 100)),
                Material::new_color_no_shade(0.5, 0.5, 0.5, 1.0),
            )),
            "Grid",
            DrawMode::Lines
        );
        grid.set_scale(50.0);
        grid.set_rotation(UnitQuaternion::from_euler_angles(PI / 2., 0., 0.));
        let gizmo = create_transform_gizmo(&scene, ArrowTip::Cone);
        let spawn_origin = rc_rcell({
            create_origin(&scene)
        });
        scene.add(spawn_origin.clone());
        scene.show(&gizmo);
        let gizmo = Gizmo::new(gizmo);
        let gizmo = rc_rcell(gizmo);
        scene.show(&grid);
        let active_node = rc_rcell(None);
        let mut editor = Self {
            scene: scene.clone(),
            gizmo,
            active_node,
            spawn_origin,
        };
        scene_tree::build(&editor);
        toolbar::build(&editor);
        editor.scale_gizmos();
        editor.add_events();
        editor
    }
    pub fn scale_gizmos(&self) {
        use crate::log;
        let eye = Point3::from(self.scene().view().borrow().eye());
        let gizmo = self.gizmo.borrow();
        let gizmo = gizmo.node();
        let origin = self.spawn_origin.borrow();
        let g_pos = Point3::from(gizmo.global_position());
        let o_pos = Point3::from(origin.global_position());
        let g_mag = (eye - g_pos).magnitude() / 60.;
        let o_mag = (eye - o_pos).magnitude() / 60.; 
        gizmo.set_scale(g_mag);
        origin.set_scale(o_mag);
    }
    pub fn set_active_node(&self, node: RcRcell<Node>) {
        self.scene.view().borrow_mut().focus(&node.borrow());
        let gizmo = self.gizmo.borrow();
        gizmo.apply_target_transform(&node.borrow());
        *self.active_node.borrow_mut() = Some(node);
        self.scale_gizmos();
    }
    fn add_events(&mut self) {
        let editor = self.clone();
        let rndr = self.scene.renderer();
        let renderer = rndr.clone();
        add_event(&rndr.borrow().canvas(), "mousedown", move |e| {
            get_el("mesh-list").class_list().remove_1("shown").unwrap();
            let me = e.dyn_into::<MouseEvent>().unwrap();

            let view = editor.scene.view();
            if view.borrow().zooming() {
                return;
            }

            let ray = Self::get_ray_from_screen(&me, &view.borrow(), renderer.borrow().canvas());

            if !editor
                .gizmo
                .borrow_mut()
                .handle_mousedown(&ray, &view.borrow())
            {
                if let Some((node, _)) = editor.scene.root().borrow().collides_w_children(&ray) {
                    editor.set_active_node(node);
                }
            }
        });

        let editor = self.clone();
        let rndr = self.scene.renderer();
        let renderer = rndr.clone();
        add_event(&rndr.borrow().canvas(), "mousemove", move |e| {
            let gizmo = editor.gizmo.borrow();
            let view = editor.scene.view();
            if gizmo.collision_constraint() == CollisionConstraint::None || view.borrow().zooming() {
                return;
            }
            let active_node = editor.active_node.borrow();
            let mut view = view.borrow_mut();
            view.disable_rotation();
            let me = e.dyn_into::<MouseEvent>().unwrap();
            let ray = Self::get_ray_from_screen(&me, &view, &renderer.borrow().canvas());
            gizmo.handle_mousemove(&ray, &active_node);
        });

        let editor = self.clone();
        add_event(&window(), "mousemove", move |e| {
            let view = editor.scene().view();
            {
                let me = e.dyn_into::<MouseEvent>().unwrap();
                view.borrow_mut().update_zoom(me.movement_y());
            }
            if view.borrow().zooming() {
                editor.scale_gizmos();
            }
        });

        let editor = self.clone();
        let rndr = self.scene.renderer();
        add_event(&rndr.borrow().canvas(), "wheel", move |_| {
            editor.scale_gizmos();
        });

        let a_gizmo = self.gizmo.clone();
        add_event(&rndr.borrow().canvas(), "mouseup", move |_| {
            let mut gizmo = a_gizmo.borrow_mut();
            if gizmo.collision_constraint() == CollisionConstraint::None {
                return;
            }
            gizmo.handle_mouseup();
        });


        let editor = self.clone();
        add_event(&window(), "keydown", move |e| {
            let view = editor.scene().view();
            let keycode = e.dyn_into::<KeyboardEvent>().unwrap().code();
            if keycode == "KeyP" {
                handle_persp_toggle(view.clone())
            } else if keycode == "KeyZ" {
                view.borrow_mut().enable_zoom();
            } else if keycode == "KeyF" {
                if let Some(node) = editor.active_node.borrow().as_ref() {
                    view.borrow_mut().focus(&node.borrow());
                    editor.scale_gizmos();
                }
            } else if keycode == "KeyR" {
                view.borrow_mut().reset();
                editor.scale_gizmos();
            } else if keycode == "KeyA" {
                get_el("mesh-list").class_list().toggle("shown").unwrap();
            }
        });
        let view = self.scene.view();
        add_event(&window(), "keyup", move |e| {
            let keycode = e.dyn_into::<KeyboardEvent>().unwrap().code();
            if keycode == "KeyZ" {
                let mut view = view.borrow_mut();
                view.disable_zoom();
            }
        });
    }
    fn scene(&self) -> Rc<Scene> {
        self.scene.clone()
    }
    fn get_ray_from_screen(
        me: &MouseEvent,
        view: &Viewport,
        canvas: &HtmlCanvasElement,
    ) -> Ray<f32> {
        let (hw, hh) = (
            (canvas.offset_width() / 2) as f32,
            (canvas.offset_height() / 2) as f32,
        );
        let (x, y) = (me.offset_x() as f32 - hw, hh - me.offset_y() as f32);
        let (x, y) = (x / hw, y / hh);
        let ray_pos = view.screen_to_world([x, y, -1.0]);
        let ray_vec = view.screen_to_ray([x, y]);
        Ray::new(ray_pos.into(), ray_vec.into())
    }
}
