use super::gizmo::{CollisionConstraint, Gizmo};
use crate::{
    dom_factory::{
        add_event, body, document, get_el, icon_btn_w_id, query_els, query_html_el, window,
        get_target_innerh,get_target_parent_el, get_target_el,
    },
    mesh::{Geometry, Material},
    rc_rcell,
    renderer::DrawMode,
    scene::{
        primitives::{create_primitive_node, create_transform_gizmo, ArrowType},
        Node, Scene,
    },
    LightType, Primitive, RcRcell, Renderer, Viewport,
};
use genmesh::generators::Plane;
use maud::{html, Markup};
use nalgebra::UnitQuaternion;
use ncollide3d::query::Ray;
use std::f32::consts::PI;
use std::rc::Rc;
use std::str::FromStr;
use strum::IntoEnumIterator;
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlCanvasElement, KeyboardEvent, MouseEvent, HtmlElement};

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
            Material::single_color_no_shade(0.5, 0.5, 0.5, 1.0),
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
    pub fn set_active_node_internal(
        a_node: RcRcell<Option<Rc<Node>>>,
        node: Rc<Node>,
        view: &mut Viewport,
        gizmo: &Gizmo,
    ) {
        view.focus(&node);
        gizmo.copy_location(&node);
        let mut active_node = a_node.borrow_mut();
        *active_node = Some(node);
    }
    pub fn set_active_node(&mut self, node: Rc<Node>) {
        let mut view = self.view.borrow_mut();
        Self::set_active_node_internal(
            self.active_node.clone(),
            node,
            &mut view,
            &self.gizmo.borrow(),
        );
    }
    fn recurse_tree(node: &Node, hidden: bool) -> Markup {
        let owned_children = node.owned_children();
        let children = node.children();
        let class_name = if hidden { "disabled" } else { "shown" };
        html! {
            ul class=(class_name) {
                li {
                    @if owned_children.len() > 0 || children.len() > 0 {
                        @let icon = if owned_children.len() > 0 {
                            "expand_more"
                        } else {
                            "expand_less"
                        };
                        i.material-icons-outlined{(icon)}
                    }
                    p {(node.info().name)}
                    @if !hidden {
                        i.material-icons {"visibility"}
                    }
                    @for child in owned_children {
                        (Self::recurse_tree(&child, true))
                    }
                    @for child in children {
                        (Self::recurse_tree(&child, false))
                    }
                }
            }
        }
    }
    fn markup(scene: &Scene) -> String {
        let markup = html! {
            section #toolbar {
                (icon_btn_w_id("add-mesh", "Add a new object", "add", "A"))
                //(icon_btn_w_id("translate", "Translate selected object", "call_merge", "G"))
                //(icon_btn_w_id("rotate", "Rotate selected object", "360", "R"))
                //(icon_btn_w_id("scale", "Scale selected object", "image_aspect_ratio", "S"))
                (icon_btn_w_id("focus", "Focus view to selected object", "center_focus_weak", "F"))
                (icon_btn_w_id("toggle-perspective", "Switch Perspective", "crop_5_4", "P"))
                (icon_btn_w_id("zoom-in-out", "Zoom in/out view", "zoom_in", "Z"))
            }
            section #mesh-list.panel {
                h3 {"Add Objects" hr{} "Mesh"}
                ul#mesh {
                    @for each in Primitive::iter() {
                        li{(each.to_string().as_str())}
                    }
                }
                h3 {"Light"}
                ul#light {
                    @for light in LightType::iter() {
                        li{(light.to_string().as_str())}
                    }
                }
            }
            section #scene-tree.panel {
                h3 {"Scene"}
                (Self::recurse_tree(scene.root(), false))
            }
        };
        markup.into_string()
    }
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
        let a_active = self.active_node.clone();
        add_event(
            &document().get_element_by_id("focus").unwrap(),
            "click",
            move |_| {
                if let Some(node) = a_active.borrow().as_ref() {
                    a_view.borrow_mut().focus(&node);
                }
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
        let list = &query_els("#mesh-list #mesh li");
        for i in 0..list.length() {
            let each = list.get(i).unwrap();
            let a_scene = self.scene.clone();
            add_event(
                &each.dyn_into::<EventTarget>().unwrap(),
                "click",
                move |e| {
                    let node = {
                        let scene = a_scene.borrow();
                        Rc::new(create_primitive_node(
                            &scene,
                            Primitive::from_str(&get_target_innerh(&e)).unwrap().into(),
                        ))
                    };
                    {
                        let mut scene = a_scene.borrow_mut();
                        scene.add(node);
                    }
                },
            );
        }
        let list = &query_els("#mesh-list #light li");
        for i in 0..list.length() {
            let each = list.get(i).unwrap();
            let a_scene = self.scene.clone();
            add_event(
                &each.dyn_into::<EventTarget>().unwrap(),
                "click",
                move |e| {
                    let mut scene = a_scene.borrow_mut();
                    let light = scene.light(
                        LightType::from_str(&get_target_innerh(&e)).unwrap(),
                        [1.0, 1.0, 1.0],
                        1.0,
                    );
                    scene.add_light(light);
                },
            );
        }
        let list = &query_els("#scene-tree p");
        for i in 0..list.length() {
            let each = list.get(i).unwrap();
            let a_scene = self.scene.clone();
            let a_node = self.active_node.clone();
            let a_gizmo = self.gizmo.clone();
            let a_view = self.view.clone();
            add_event(
                &each.dyn_into::<EventTarget>().unwrap(),
                "click",
                move |e| {
                    let scene = a_scene.borrow();
                    let mut view = a_view.borrow_mut();
                    if let Some(node) = scene.find_node_w_name(&get_target_innerh(&e)) {
                        Self::set_active_node_internal(
                            a_node.clone(),
                            node.clone(),
                            &mut view,
                            &a_gizmo.borrow(),
                        );
                    }
                },
            );
        }
        let list = &query_els("#scene-tree i");
        for i in 0..list.length() {
            let each = list.get(i).unwrap();
            let a_scene = self.scene.clone();
            add_event(
                &each.dyn_into::<EventTarget>().unwrap(),
                "click",
                move |e| {
                    let scene = a_scene.borrow();
                    match get_target_innerh(&e).as_str() {
                        "expand_more" => {
                            let children = get_target_parent_el(&e,1).children();
                            for i in 2..children.length() {
                                children.item(i).unwrap().class_list().add_1("shown");
                            }
                            get_target_el(&e).set_inner_html("expand_less");
                        }
                        "expand_less" => {
                            let children = get_target_parent_el(&e,1).children();
                            for i in 2..children.length() {
                                children.item(i).unwrap().class_list().remove_1("shown");
                            }
                            get_target_el(&e).set_inner_html("expand_more");
                        }
                        "visibility"=> {
                            get_target_el(&e).set_inner_html("visibility_off");
                            let name = get_target_el(&e).previous_sibling().unwrap().dyn_into::<HtmlElement>().unwrap().inner_html();
                            if let Some(node) = scene.find_node_w_name(&name) {
                                scene.hide_only(&node);
                            }
                        }
                        "visibility_off"=> {
                            get_target_el(&e).set_inner_html("visibility");
                            let name = get_target_el(&e).previous_sibling().unwrap().dyn_into::<HtmlElement>().unwrap().inner_html();
                            if let Some(node) = scene.find_node_w_name(&name) {
                                scene.show_only(&node);
                            }
                        }
                        _=>()
                    }
                },
            );
        }

        let a_scene = self.scene.clone();
        let a_rndr = self.renderer.clone();
        let a_view = self.view.clone();
        let a_gizmo = self.gizmo.clone();
        let a_node = self.active_node.clone();
        add_event(self.renderer.borrow().canvas(), "mousedown", move |e| {
            let mut view = a_view.borrow_mut();
            let scene = a_scene.borrow();

            get_el("mesh-list").class_list().remove_1("shown").unwrap();
            let me = e.dyn_into::<MouseEvent>().unwrap();

            let renderer = a_rndr.borrow_mut();
            let ray = Self::get_ray_from_screen(&me, &view, renderer.canvas());

            if !a_gizmo.borrow_mut().handle_mousedown(&ray, &view) {
                if let Some((node, _)) = scene.root().collides_w_children(&ray) {
                    Self::set_active_node_internal(
                        a_node.clone(),
                        node.clone(),
                        &mut view,
                        &a_gizmo.borrow(),
                    );
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
        let a_gizmo = self.gizmo.clone();
        add_event(&window(), "mousemove", move |e| {
            let me = e.dyn_into::<MouseEvent>().unwrap();
            let mut view = a_view.borrow_mut();
            view.update_zoom(me.movement_y());
            if view.zooming() {
                a_gizmo
                    .borrow()
                    .rescale(view.transform().translation.vector.magnitude());
            }
        });

        let a_view = self.view.clone();
        let a_node = self.active_node.clone();
        add_event(&window(), "keydown", move |e| {
            let keycode = e.dyn_into::<KeyboardEvent>().unwrap().code();
            if keycode == "KeyP" {
                handle_persp_toggle(a_view.clone());
            } else if keycode == "KeyZ" {
                let mut view = a_view.borrow_mut();
                view.enable_zoom();
            } else if keycode == "KeyF" {
                let mut view = a_view.borrow_mut();
                if let Some(node) = a_node.borrow().as_ref() {
                    view.focus(&node);
                }
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
