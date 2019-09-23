mod console;
pub use console::console_setup;

use crate::{
    dom_factory::{
        add_event, body, document, get_el, icon_btn_w_id, query_els, query_html_el, window,
    },
    mesh::{Geometry, Material, multiply},
    rc_rcell,
    renderer::DrawMode,
    scene::{
        primitives::{create_primitive_node, create_transform_gizmo, ArrowType, GizmoGrab},
        Gizmo, Node, Scene,
    },
    RcRcell, Renderer, Viewport,
};
use genmesh::generators::{IcoSphere, Plane};
use maud::html;
use nalgebra::{Isometry3, Translation3, UnitQuaternion, Unit, Vector3, Point3};
use std::f32::consts::PI;
//use std::cell::RefCell;
//use std::rc::Rc;
use ncollide3d::{
    query::Ray,
    query::RayCast,
    shape::{Ball, ConvexHull, Cuboid, Plane as CollidePlane},
};
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlCanvasElement, HtmlElement, KeyboardEvent, MouseEvent};

pub struct Editor {
    view: RcRcell<Viewport>,
    scene: RcRcell<Scene>,
    renderer: RcRcell<Renderer>,
    gizmo: RcRcell<Gizmo>,
}

impl Editor {
    pub fn new(
        view: RcRcell<Viewport>,
        scene: RcRcell<Scene>,
        renderer: RcRcell<Renderer>,
    ) -> Self {
        body()
            .insert_adjacent_html("beforeend", Self::markup().as_str())
            .expect("Couldn't insert console into the DOM!");
        let gizmo = {
            let scene = scene.borrow_mut();
            let renderer = renderer.borrow();
            let grid = scene.object_from_mesh_and_name(
                Geometry::from_genmesh_no_normals(&Plane::subdivide(100, 100)),
                Material::single_color_no_shade(0.8, 0.8, 0.8, 1.0),
                "Grid",
            );
            grid.scale(50.0);
            grid.set_rotation(UnitQuaternion::from_euler_angles(PI / 2., 0., 0.));
            scene.add_with_mode(&grid, DrawMode::Lines);
            let gizmo = create_transform_gizmo(&scene, ArrowType::Cone);
            scene.add(&gizmo);
            gizmo
        };
        let gizmo = rc_rcell(Gizmo::Translate(gizmo, GizmoGrab::None));
        let mut editor = Self {
            view,
            scene,
            renderer,
            gizmo,
        };
        editor.add_events();
        editor
    }
    fn markup() -> String {
        let markup = html! {
            section #toolbar {
                (icon_btn_w_id("add-mesh", "Add a new object", "add", "A"))
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
        };
        markup.into_string()
    }
    fn get_ray_from_screen (me: &MouseEvent, view: &Viewport, canvas: &HtmlCanvasElement) -> Ray<f32> {
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
    fn ray_intersects_with_pan_cuboid (node: &Node, ray: &Ray<f32>) -> bool {
        let c_t = node.get_transform();
        let p_t = node.get_parent_transform();
        let s = multiply(c_t.scale,p_t.scale);
        let verts: Vec<Point3<f32>> = node.get_mesh().unwrap()
                                .geometry.vertices.chunks(3)
                                .map(|c| Point3::new(c[0] * s.x, c[1] * s.y, c[2] * s.z))
                                .collect();
        if let Some(target) = ConvexHull::try_from_points(&verts) {
            let mut transform = c_t.isometry * p_t.isometry;
            transform.translation.vector = multiply(transform.translation.vector, s); 
            log!("TRANSFORM", c_t.isometry.translation.vector,p_t.isometry.translation.vector, transform);
            if target.intersects_ray(&transform, &ray) {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    fn set_state_and_change_color (node: &Node, g_state: &mut GizmoGrab, color: [f32;3], g_new_state: GizmoGrab) {
        *g_state = g_new_state;
        let mut mesh = node.get_mesh().unwrap();
        mesh.material = Material::single_color_no_shade(color[0], color[1], color[2], 1.);
        node.set_mesh(mesh);
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
            let a_rndr = self.renderer.clone();
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
                        create_primitive_node(&scene, selected_prim.as_str().into())
                    };
                    {
                        let scene = a_scene.borrow_mut();
                        scene.add(&node);
                    }
                    let mut renderer = a_rndr.borrow_mut();
                    let scene = a_scene.borrow();
                },
            );
        }


        let a_rndr = self.renderer.clone();
        let a_view = self.view.clone();
        let a_gizmo = self.gizmo.clone();
        let a_scene = self.scene.clone();
        add_event(self.renderer.borrow().canvas(), "mousedown", move |e| {
            let mut renderer = a_rndr.borrow_mut();
            let mut view = a_view.borrow_mut();
            let mut gizmo = a_gizmo.borrow_mut();
            let scene = a_scene.borrow();

            get_el("mesh-list").class_list().remove_1("shown").unwrap();
            let me = e.dyn_into::<MouseEvent>().unwrap();

            let (gizmo_node, gizmo_state) = gizmo.inner_mut();
            let transform = gizmo_node.get_transform();
            let target = Ball::new(0.5);

            let ray = Self::get_ray_from_screen(&me, &view, renderer.canvas());
            gizmo_node.set_position([-2.0,0.,0.]);

            if target.intersects_ray(&transform.isometry, &ray) {
                Self::set_state_and_change_color(gizmo_node, gizmo_state, [1.,1.,1.], GizmoGrab::ViewPlane);
            } else {
                for child in gizmo_node.owned_children() {
                    match child.get_info().name.as_str() {
                        "pan_x" => {
                            if Self::ray_intersects_with_pan_cuboid(&child, &ray) {
                                Self::set_state_and_change_color(&child, gizmo_state, [1.,0.,0.], GizmoGrab::XPlane);
                                break;
                            }
                        },
                        "pan_y" => {
                            if Self::ray_intersects_with_pan_cuboid(&child, &ray) {
                                Self::set_state_and_change_color(&child, gizmo_state, [0.,1.,0.], GizmoGrab::YPlane);
                                break;
                            }
                        },
                        "pan_z" => {
                            if Self::ray_intersects_with_pan_cuboid(&child, &ray) {
                                Self::set_state_and_change_color(&child, gizmo_state, [0.,0.,1.], GizmoGrab::ZPlane);
                                break;
                            }
                        },
                        _=>()
                    };
                }
            }
        });

        let a_view = self.view.clone();
        let a_rndr = self.renderer.clone();
        let a_gizmo = self.gizmo.clone();
        let renderer = self.renderer.clone();
        add_event(self.renderer.borrow().canvas(), "mousemove", move |e| {
            let renderer = a_rndr.borrow();
            let canvas = renderer.canvas();
            let mut view = a_view.borrow_mut();
            let mut gizmo = a_gizmo.borrow_mut();
            let (gizmo_node, gizmo_state) = gizmo.inner_mut();

            if *gizmo_state == GizmoGrab::None {
                return;
            }

            let me = e.dyn_into::<MouseEvent>().unwrap();
            let (axis, transform) = match gizmo_state {
                GizmoGrab::ViewPlane => {
                    (
                        Vector3::z_axis(),
                        Isometry3::from_parts(
                            Translation3::new(0., 0., 0.),
                            view.get_transform().inverse().rotation
                        )
                    )
                },
                GizmoGrab::XPlane => (Vector3::x_axis(), Isometry3::identity()),
                GizmoGrab::YPlane => (Vector3::y_axis(), Isometry3::identity()),
                _ => (Vector3::z_axis(), Isometry3::identity()),
            };
            view.disable_rotation();
            let ray = Self::get_ray_from_screen(&me, &view, &canvas);
            let pan_view = CollidePlane::new(axis);
            if let Some(i) = pan_view.toi_and_normal_with_ray(&transform, &ray, true) {
                let pos = ray.point_at(i.toi);
                //gizmo_node.set_position([pos.x, pos.y, pos.z]);
            }
        });

        let a_gizmo = self.gizmo.clone();
        add_event(self.renderer.borrow().canvas(), "mouseup", move |e| {
            let mut gizmo = a_gizmo.borrow_mut();
            let (gizmo_node, gizmo_state) = gizmo.inner_mut();
            if *gizmo_state == GizmoGrab::ViewPlane {
                Self::set_state_and_change_color(gizmo_node, gizmo_state, [0.8,0.8,0.8], GizmoGrab::None);
            } else {
                for child in gizmo_node.owned_children() {
                    match child.get_info().name.as_str() {
                        "pan_x" => {
                            if *gizmo_state == GizmoGrab::XPlane {
                                Self::set_state_and_change_color(&child, gizmo_state, [0.8,0.,0.], GizmoGrab::None);
                            }
                        },
                        "pan_y" => {
                            if *gizmo_state == GizmoGrab::YPlane {
                                Self::set_state_and_change_color(&child, gizmo_state, [0.,0.8,0.], GizmoGrab::None);
                            }
                        },
                        "pan_z" => {
                            if *gizmo_state == GizmoGrab::ZPlane {
                                Self::set_state_and_change_color(&child, gizmo_state, [0.,0.,0.8], GizmoGrab::None);
                            }
                        }, _=>()
                    }
                }
            }
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
            let dy = if me.movement_y() > 0 { 0.1 } else { -0.1 };
            view.update_zoom(dy);
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
}
