mod console;
mod gizmo;
use crate::{
    dom_factory::{
        add_class, add_event, body, create_el, create_el_w_class_n_inner, document, el_innerh,
        get_el, get_parent, get_target_el, get_target_innerh, get_target_parent_el, icon_btn_w_id,
        insert_el, insert_el_at, query_el, query_els, query_html_el, remove_class, window,
    },
    mesh::{Geometry, Material},
    rc_rcell,
    renderer::DrawMode,
    scene::{
        primitives::{create_origin, create_primitive_node, create_transform_gizmo, ArrowTip},
        Node, Scene,
    },
    LightType, Primitive, RcRcell, Renderer, Viewport,
};
pub use console::{console_setup, ConsoleConfig};
use genmesh::generators::Plane;
pub use gizmo::{CollisionConstraint, Gizmo};
use maud::html;
use nalgebra::UnitQuaternion;
use ncollide3d::query::Ray;
use std::f32::consts::PI;
use std::rc::Rc;
use std::str::FromStr;
use strum::IntoEnumIterator;
use wasm_bindgen::JsCast;
use web_sys::{
    DataTransfer, DragEvent, Element, EventTarget, HtmlCanvasElement, HtmlElement, KeyboardEvent,
    MouseEvent,
};

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
        let node = create_transform_gizmo(&scene, ArrowTip::Cone);
        let spawn_origin = rc_rcell({
            let o = create_origin(&scene);
            o.set_scale(0.2);
            o
        });
        scene.add(spawn_origin.clone());
        scene.show(&node);
        let gizmo = Gizmo::new(node);
        gizmo.rescale(&scene.view().borrow().transform());
        let gizmo = rc_rcell(gizmo);
        scene.show(&grid);
        let active_node = rc_rcell(None);
        body()
            .insert_adjacent_html("beforeend", Self::markup().as_str())
            .expect("Couldn't insert console into the DOM!");
        let mut editor = Self {
            scene: scene.clone(),
            gizmo,
            active_node,
            spawn_origin,
        };
        editor.markup_node(&get_el("scene-tree"), NodeRef::Mutable(scene.root()));
        editor.add_events();
        editor
    }
    pub fn track_gizmo(&self) {
        if let Some(node) = self.active_node.borrow().as_ref() {
            let node = node.borrow();
            let gizmo = self.gizmo.borrow();
            self.scene.view().borrow_mut().focus(&node);
            gizmo.apply_target_transform(&node);
        }
    }
    pub fn set_active_node(&self, node: RcRcell<Node>) {
        *self.active_node.borrow_mut() = Some(node);
        self.gizmo
            .borrow()
            .rescale(&self.scene.view().borrow().transform());
        self.track_gizmo();
    }
    fn markup() -> String {
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
            section #right-panel {
                section #scene-tree.panel {
                    h3 {"Scene"}
                }
                section #properties.panel {
                    h3 {"Properties"}
                }
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
        let a_view = self.scene.view();
        add_event(
            &document().get_element_by_id("toggle-perspective").unwrap(),
            "click",
            move |_| {
                handle_persp_toggle(a_view.clone());
            },
        );
        let a_view = self.scene.view();
        let a_active = self.active_node.clone();
        add_event(
            &document().get_element_by_id("focus").unwrap(),
            "click",
            move |_| {
                if let Some(node) = a_active.borrow().as_ref() {
                    a_view.borrow_mut().focus(&node.borrow());
                }
            },
        );
        let a_view = self.scene.view();
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
            let editor = self.clone();
            add_event(
                &each.dyn_into::<EventTarget>().unwrap(),
                "click",
                move |e| {
                    let scene = &editor.scene;
                    let node = rc_rcell(create_primitive_node(
                        scene,
                        Primitive::from_str(&get_target_innerh(&e)).unwrap(),
                    ));
                    node.borrow().copy_location(&editor.spawn_origin.borrow());
                    editor.markup_node(
                        &query_el("#scene-tree > ul"),
                        NodeRef::Mutable(node.clone()),
                    );
                    scene.add(node);
                },
            );
        }
        let list = &query_els("#mesh-list #light li");
        for i in 0..list.length() {
            let each = list.get(i).unwrap();
            let editor = self.clone();
            add_event(
                &each.dyn_into::<EventTarget>().unwrap(),
                "click",
                move |e| {
                    let scene = &editor.scene;
                    let light = scene.light(
                        LightType::from_str(&get_target_innerh(&e)).unwrap(),
                        [1.0, 1.0, 1.0],
                        1.0,
                    );
                    light
                        .node()
                        .borrow()
                        .copy_location(&editor.spawn_origin.borrow());
                    scene.add_light(&light);
                    editor.markup_node(
                        &query_el("#scene-tree > ul"),
                        NodeRef::Mutable(light.node().clone()),
                    );
                },
            );
        }

        let editor = self.clone();
        let rndr = self.scene.renderer();
        let renderer = rndr.clone();
        add_event(&rndr.borrow().canvas(), "mousedown", move |e| {
            get_el("mesh-list").class_list().remove_1("shown").unwrap();
            let me = e.dyn_into::<MouseEvent>().unwrap();

            let view = editor.scene.view();
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
            let active_node = editor.active_node.borrow();
            let gizmo = editor.gizmo.borrow();
            let view = editor.scene.view();
            let mut view = view.borrow_mut();
            if gizmo.collision_constraint() == CollisionConstraint::None {
                return;
            }
            view.disable_rotation();
            let me = e.dyn_into::<MouseEvent>().unwrap();
            let ray = Self::get_ray_from_screen(&me, &view, &renderer.borrow().canvas());
            gizmo.handle_mousemove(&ray, &active_node);
        });

        let a_gizmo = self.gizmo.clone();
        let a_view = self.scene.view();
        let rndr = self.scene.renderer();
        add_event(&rndr.borrow().canvas(), "wheel", move |_| {
            let view = a_view.borrow();
            a_gizmo.borrow().rescale(&view.transform());
        });

        let a_gizmo = self.gizmo.clone();
        add_event(&rndr.borrow().canvas(), "mouseup", move |_| {
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

        let a_view = self.scene.view();
        let a_gizmo = self.gizmo.clone();
        add_event(&window(), "mousemove", move |e| {
            let me = e.dyn_into::<MouseEvent>().unwrap();
            let mut view = a_view.borrow_mut();
            view.update_zoom(me.movement_y());
            if view.zooming() {
                a_gizmo.borrow().rescale(&view.transform());
            }
        });

        let a_view = self.scene.view();
        let a_gizmo = self.gizmo.clone();
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
                    view.focus(&node.borrow());
                }
            } else if keycode == "KeyR" {
                let mut view = a_view.borrow_mut();
                view.reset();
                a_gizmo.borrow().rescale(&view.transform());
            } else if keycode == "KeyA" {
                get_el("mesh-list").class_list().toggle("shown").unwrap();
            }
        });
        let a_view = self.scene.view();
        add_event(&window(), "keyup", move |e| {
            let keycode = e.dyn_into::<KeyboardEvent>().unwrap().code();
            if keycode == "KeyZ" {
                let mut view = a_view.borrow_mut();
                view.disable_zoom();
            }
        });
    }
    pub fn markup_node(&self, parent: &Element, node: NodeRef) -> Element {
        let p = create_el("p");
        let li = create_el("li");
        insert_el(&li, &p);
        let ul = create_el("ul");
        let add_collapse_icon = |children: &Vec<RcRcell<Node>>, owned_children: &Vec<Node>| {
            if !children.is_empty() || !owned_children.is_empty() {
                let icon = if owned_children.is_empty() {
                    "expand_less"
                } else {
                    "expand_more"
                };
                let i = create_el_w_class_n_inner("i", "material-icons fold foldable", icon);
                insert_el(&li, &i);
            } else {
                let i = create_el_w_class_n_inner("i", "material-icons fold", "control_camera");
                insert_el(&li, &i);
            }
        };
        let editor = self.clone();
        let recurse_children = |children: &Vec<RcRcell<Node>>, owned_children: &Vec<Node>| {
            for child in children {
                let child_el = editor.markup_node(&ul, NodeRef::Mutable(child.clone()));
                let li = create_el("li");
                insert_el(&li, &child_el);
                insert_el(&ul, &li);
            }
            for child in owned_children {
                let child_el = editor.markup_node(&ul, NodeRef::Owned(child));
                Self::handle_node_folding(&child_el);
                let li = create_el("li");
                insert_el(&li, &child_el);
                insert_el(&ul, &li);
            }
        };
        insert_el(&ul, &li);
        let name = match node {
            NodeRef::Mutable(n) => {
                let node = n.borrow();
                let (children, owned_children) = (node.children(), node.owned_children());
                add_collapse_icon(children, owned_children);
                if parent.id().as_str() != "scene-tree" {
                    let eyei = create_el_w_class_n_inner("i", "material-icons eye", "visibility");
                    insert_el(&li, &eyei);
                    editor.add_node_events(&ul, n.clone());
                } else {
                    Self::handle_node_folding(&ul);
                }
                editor.add_drag_events(&p);
                let name = node.info().name;
                add_class(&ul, "shown");
                recurse_children(children, owned_children);
                p.set_attribute("draggable", "true").unwrap();
                name
            }
            NodeRef::Owned(n) => {
                let (children, owned_children) = (n.children(), n.owned_children());
                add_collapse_icon(children, owned_children);
                add_class(&ul, "disabled");
                recurse_children(children, owned_children);
                n.info().name
            }
        };
        let p = p.dyn_into::<HtmlElement>().unwrap();
        p.set_inner_html(name.as_str());
        insert_el(&parent, &ul);
        ul
    }

    pub fn handle_node_folding(el: &Element) {
        let el = el.children().item(0).unwrap().children().item(1).unwrap();
        if el.class_list().contains("foldable") {
            add_event(&el, "click", move |e| {
                let icon = get_target_innerh(&e);
                let children = get_target_parent_el(&e, 2).children();
                for i in 1..children.length() {
                    let class_list = children
                        .item(i)
                        .unwrap()
                        .children()
                        .item(0)
                        .unwrap()
                        .class_list();
                    match icon.as_str() {
                        "expand_more" => {
                            class_list.add_1("shown").unwrap();
                        }
                        "expand_less" => {
                            class_list.remove_1("shown").unwrap();
                        }
                        _ => (),
                    }
                }
                let next_icon = match icon.as_str() {
                    "expand_more" => "expand_less",
                    _ => "expand_more",
                };
                get_target_el(&e).set_inner_html(next_icon);
            });
        }
    }
    pub fn add_drag_events(&self, el: &Element) {
        add_event(el, "dragenter", move |e| {
            add_class(&get_target_el(&e), "dragenter");
        });
        add_event(el, "dragleave", move |e| {
            remove_class(&get_target_el(&e), "dragenter");
        });
        add_event(el, "dragstart", move |e| {
            add_class(&get_target_el(&e), "dragged-el");
        });
        add_event(el, "dragend", move |e| {
            remove_class(&get_target_el(&e), "dragged-el");
        });
        add_event(el, "dragover", |e| {
            e.prevent_default();
        });
        let a_editor = self.clone();
        add_event(el, "drop", move |e| {
            remove_class(&get_target_el(&e), "dragenter");
            let dragged_el = query_el("#scene-tree p.dragged-el");
            let dragged_el_name = el_innerh(dragged_el.clone());
            let dragged_parent_el = get_parent(&dragged_el, 4).unwrap();
            let dragged_parent_name = el_innerh(
                dragged_parent_el
                    .children()
                    .item(0)
                    .unwrap()
                    .children()
                    .item(0)
                    .unwrap(),
            );
            log!(dragged_parent_name);
            let drop_target_name = get_target_innerh(&e);
            if drop_target_name != dragged_el_name && drop_target_name != dragged_parent_name {
                log!("Dropping");
                let scene = &a_editor.scene;
                log!(dragged_el_name dragged_parent_name drop_target_name);
                let dragged_node = scene.find_node_w_name(&dragged_el_name).unwrap();
                let parent_node = scene.find_node_w_name(&dragged_parent_name).unwrap();
                let target_node = scene.find_node_w_name(&drop_target_name).unwrap();
                parent_node.borrow_mut().remove(&dragged_el_name);
                target_node.borrow_mut().add(dragged_node.clone());
                if dragged_parent_name.as_str() != "root" {
                    let li = create_el("li");
                    let g_p_el = get_parent(&dragged_el, 6).unwrap();
                    a_editor.markup_node(&li, NodeRef::Mutable(parent_node));
                    insert_el(&g_p_el, &li);
                } else {
                    // this is direct children of root
                    a_editor.markup_node(
                        &get_parent(&dragged_el, 5).unwrap(),
                        NodeRef::Mutable(parent_node),
                    );
                }
                let li = create_el("li");
                a_editor.markup_node(&li, NodeRef::Mutable(dragged_node));
                insert_el_at(&get_target_parent_el(&e, 1), &li, "afterend");
                get_parent(&dragged_el, 4).unwrap().remove();
            }
        });
    }
    fn get_title_els(el: &Element) -> (Element, Element) {
        let children = el.children().item(0).unwrap().children();
        (children.item(0).unwrap(), children.item(2).unwrap())
    }
    pub fn add_node_events(&self, el: &Element, node: RcRcell<Node>) {
        let (p, eyei) = Self::get_title_els(el);
        Self::handle_node_folding(&el);
        let a_node = node.clone();
        let a_editor = self.clone();
        add_event(&p, "click", move |_| {
            a_editor.set_active_node(a_node.clone());
        });
        let a_node = node.clone();
        let scene = self.scene.clone();
        add_event(&eyei, "click", move |e| {
            match get_target_innerh(&e).as_str() {
                "visibility" => {
                    get_target_el(&e).set_inner_html("visibility_off");
                    scene.hide_only(&a_node.borrow());
                    scene.turn_lights_off(&a_node.borrow());
                }
                "visibility_off" => {
                    get_target_el(&e).set_inner_html("visibility");
                    scene.show_only(&a_node.borrow());
                    scene.turn_lights_on(&a_node.borrow());
                }
                _ => (),
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
