mod console;
mod gizmo;
pub use console::{console_setup, ConsoleConfig};
pub use gizmo::{CollisionConstraint, Gizmo};

use crate::{
    dom_factory::{
        add_event, body, document, get_el, icon_btn_w_id, query_els, query_html_el, window,
    },
    mesh::{Geometry, Material},
    rc_rcell,
    renderer::DrawMode,
    scene::{
        primitives::{create_primitive_node, create_transform_gizmo, ArrowType},
        Node, Scene,
    },
    RcRcell, Renderer, Viewport,
};
use genmesh::generators::Plane;
use maud::{html, Markup};
use nalgebra::UnitQuaternion;
use ncollide3d::query::Ray;
use std::f32::consts::PI;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlCanvasElement, HtmlElement, KeyboardEvent, MouseEvent};

/// The main GUI editor that faciliates buttons to manipulate the scene, displays log in a separate
/// window, and displays the scene tree.
pub struct Editor {
    view: RcRcell<Viewport>,
    scene: RcRcell<Scene>,
    renderer: RcRcell<Renderer>,
    gizmo: RcRcell<Gizmo>,
    active_node: RcRcell<Option<Rc<Node>>>,
}

impl Editor {
    pub fn new(
        view: RcRcell<Viewport>,
        scene: RcRcell<Scene>,
        renderer: RcRcell<Renderer>,
    ) -> Self {
        let sc = scene.borrow();
        body()
            .insert_adjacent_html("beforeend", Self::markup(&sc).as_str())
            .expect("Couldn't insert console into the DOM!");
        let grid = sc.object_from_mesh_name_and_mode(
            Geometry::from_genmesh_no_normals(&Plane::subdivide(100, 100)),
            Material::single_color_no_shade(0.8, 0.8, 0.8, 1.0),
            "Grid",
            DrawMode::Lines,
        );
        grid.set_scale(50.0);
        grid.set_rotation(UnitQuaternion::from_euler_angles(PI / 2., 0., 0.));
        let node = create_transform_gizmo(&sc, ArrowType::Cone);
        sc.show(&node);
        let gizmo = Gizmo::new(node);
        gizmo.rescale(view.borrow().transform().translation.vector.magnitude());
        let gizmo = rc_rcell(gizmo);
        sc.show(&grid);
        let active_node = rc_rcell(None);
        let mut editor = Self {
            view,
            scene: scene.clone(),
            renderer,
            gizmo,
            active_node,
        };
        editor.add_events();
        editor
    }
    pub fn track_gizmo(&mut self) {
        if let Some(node) = self.active_node.borrow().as_ref() {
            self.view.borrow_mut().focus(&node);
            self.gizmo.borrow().copy_location(&node);
        }
    }
    pub fn set_active_node_internal(a_node: RcRcell<Option<Rc<Node>>>, node: Rc<Node>) {
        let mut active_node = a_node.borrow_mut();
        *active_node = Some(node)
    }
    pub fn set_active_node(&mut self, node: Rc<Node>) {
        let mut view = self.view.borrow_mut();
        view.focus(&node);
        self.gizmo.borrow().copy_location(&node);
        Self::set_active_node_internal(self.active_node.clone(), node);
    }
    fn recurse_tree(root: &Node) -> Markup {
        let owned_children = root.owned_children();
        let children = root.children();
        html! {
            li {
                p {(root.info().name)}
                @if owned_children.len() > 0 {
                    ul {
                        @for child in owned_children {
                            (Self::recurse_tree(&child))
                        }
                    }
                }
                @if children.len() > 0 {
                    ul {
                        @for child in children {
                            (Self::recurse_tree(&child))
                        }
                    }
                }
            }
        }
    }
    fn markup(scene: &Scene) -> String {
        let markup = html! {
            section #toolbar {
                (icon_btn_w_id("add-mesh", "Add a new object", "add", "A"))
                (icon_btn_w_id("translate", "Translate selected object", "call_merge", "G"))
                (icon_btn_w_id("rotate", "Rotate selected object", "360", "R"))
                (icon_btn_w_id("scale", "Scale selected object", "image_aspect_ratio", "S"))
                (icon_btn_w_id("transform", "Translate/Scale/Rotate selected object", "crop_rotate", "T"))
                (icon_btn_w_id("toggle-perspective", "Switch Perspective", "crop_5_4", "P"))
                (icon_btn_w_id("zoom-in-out", "Zoom in/out view", "zoom_in", "Z"))
            }
            section #mesh-list {
                h3 {"Add Mesh"}
                ul {
                    li{"Plane"}
                    li{"Cube"}
                    li{"Circle"}
                    li{"Cylinder"}
                    li{"Cone"}
                    li{"UVSphere"}
                    li{"IcoSphere"}
                    li{"Torus"}
                }
            }
            section #scene-tree {
                ul {
                    (Self::recurse_tree(scene.root()))
                }
            }
        };
        markup.into_string()
    }
    //pub fn update(&mut self, view: &mut Viewport) {
    //let node = self.active_node.borrow();
    //if let Some(node) = node.as_ref() {
    //view.focus(&node);
    //let gizmo = self.gizmo.borrow();
    //let gizmo_node = gizmo.inner().0;
    //let p_t = node.parent_transform();
    //gizmo_node.set_parent_transform(p_t);
    //let v = node.position();
    //gizmo_node.set_position(v.x, v.y, v.z);
    //let ds =
    //1. / p_t.scale.magnitude() * view.transform().translation.vector.magnitude() / 20.;
    //gizmo_node.set_scale(ds);
    ////log!(node.borrow().info());
    //}
    //}
    fn add_events(&mut self) {
        let handle_persp_toggle = |a_view: RcRcell<Viewport>| {
            let icon = query_html_el("#toggle-perspective .material-icons-outlined");
            if icon.inner_html() == "panorama_horizontal" {
                icon.set_inner_html("crop_5_4");
            } else {
                icon.set_inner_html("panorama_horizontal");
            }
            let mut view = a_view.borrow_mut();
            view.switch_projection();
        };
        let a_view = self.view.clone();
        add_event(
            &document().get_element_by_id("toggle-perspective").unwrap(),
            "click",
            move |_| {
                handle_persp_toggle(a_view.clone());
            },
        );
        let a_view = self.view.clone();
        add_event(
            &document().get_element_by_id("zoom-in-out").unwrap(),
            "mousedown",
            move |_| {
                let mut view = a_view.borrow_mut();
                view.enable_zoom();
            },
        );
        let list = &query_els("#mesh-list li");
        for i in 0..list.length() {
            let each = list.get(i).unwrap();
            let a_scene = self.scene.clone();
            add_event(
                &each.dyn_into::<EventTarget>().unwrap(),
                "click",
                move |e| {
                    let node = {
                        let scene = a_scene.borrow();
                        let selected_prim = e
                            .target()
                            .unwrap()
                            .dyn_into::<HtmlElement>()
                            .unwrap()
                            .inner_html();
                        Rc::new(create_primitive_node(&scene, selected_prim.as_str().into()))
                    };
                    {
                        let mut scene = a_scene.borrow_mut();
                        scene.add(node);
                    }
                },
            );
        }

        let a_scene = self.scene.clone();
        let a_rndr = self.renderer.clone();
        let a_view = self.view.clone();
        let a_gizmo = self.gizmo.clone();
        add_event(self.renderer.borrow().canvas(), "mousedown", move |e| {
            let mut view = a_view.borrow_mut();
            let scene = a_scene.borrow();

            get_el("mesh-list").class_list().remove_1("shown").unwrap();
            let me = e.dyn_into::<MouseEvent>().unwrap();

            let renderer = a_rndr.borrow_mut();
            let ray = Self::get_ray_from_screen(&me, &view, renderer.canvas());

            if !a_gizmo.borrow_mut().handle_mousedown(&ray, &view) {
                if let Some((node, _)) = scene.root().collides_w_children(&ray) {
                    view.focus(&node);
                    a_gizmo.borrow_mut().copy_location(&node);
                }
            }
        });

        let a_view = self.view.clone();
        let a_rndr = self.renderer.clone();
        let a_gizmo = self.gizmo.clone();
        let a_node = self.active_node.clone();
        add_event(self.renderer.borrow().canvas(), "mousemove", move |e| {
            let renderer = a_rndr.borrow();
            let canvas = renderer.canvas();
            let mut view = a_view.borrow_mut();
            let active_node = a_node.borrow();
            let gizmo = a_gizmo.borrow();

            if gizmo.collision_constraint() == CollisionConstraint::None {
                return;
            }
            view.disable_rotation();
            let me = e.dyn_into::<MouseEvent>().unwrap();
            let ray = Self::get_ray_from_screen(&me, &view, &canvas);
            gizmo.handle_mousemove(&ray, &active_node);
        });

        let a_gizmo = self.gizmo.clone();
        let a_view = self.view.clone();
        add_event(self.renderer.borrow().canvas(), "wheel", move |_| {
            let view = a_view.borrow();
            a_gizmo
                .borrow()
                .rescale(view.transform().translation.vector.magnitude());
        });

        let a_gizmo = self.gizmo.clone();
        add_event(self.renderer.borrow().canvas(), "mouseup", move |_| {
            let mut gizmo = a_gizmo.borrow_mut();
            if gizmo.collision_constraint() == CollisionConstraint::None {
                return;
            }
            gizmo.handle_mouseup();
        });

        add_event(
            &document().get_element_by_id("add-mesh").unwrap(),
            "click",
            move |_| {
                get_el("mesh-list").class_list().toggle("shown").unwrap();
            },
        );
        let a_view = self.view.clone();
        add_event(&window(), "mousemove", move |e| {
            let me = e.dyn_into::<MouseEvent>().unwrap();
            let mut view = a_view.borrow_mut();
            view.update_zoom(me.movement_y());
        });

        let a_view = self.view.clone();
        add_event(&window(), "keydown", move |e| {
            let keycode = e.dyn_into::<KeyboardEvent>().unwrap().code();
            if keycode == "KeyP" {
                handle_persp_toggle(a_view.clone());
            } else if keycode == "KeyZ" {
                let mut view = a_view.borrow_mut();
                view.enable_zoom();
            } else if keycode == "KeyA" {
                get_el("mesh-list").class_list().toggle("shown").unwrap();
            }
        });
        let a_view = self.view.clone();
        add_event(&window(), "keyup", move |e| {
            let keycode = e.dyn_into::<KeyboardEvent>().unwrap().code();
            if keycode == "KeyZ" {
                let mut view = a_view.borrow_mut();
                view.disable_zoom();
            }
        });
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
