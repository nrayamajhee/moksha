pub mod console;

use crate::{
    dom_factory::{
        add_event, body, document, get_el, icon_btn_w_id, query_els, query_html_el, window,
    },
    mesh::{Geometry, Material},
    renderer::DrawMode,
    scene::{Node, Scene,primitives::{ArrowType,create_primitive_node, create_transform_gizmo}},
    Renderer, Viewport,
    RcRcell, rc_rcell,
};
use genmesh::generators::{Plane, IcoSphere};
use maud::html;
use nalgebra::UnitQuaternion;
use std::f32::consts::PI;
//use std::cell::RefCell;
//use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlElement, KeyboardEvent, MouseEvent};

pub struct Editor {
    view: RcRcell<Viewport>,
    scene: RcRcell<Scene>,
    renderer: RcRcell<Renderer>,
    gizmo: RcRcell<Node>,
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
            let cube = scene.object_from_mesh_and_name(
                Geometry::from_genmesh(&genmesh::generators::Cube::new()),
                Material::single_color(0.8, 0.8, 0.8, 1.0),
                "Cube",
            );
            let gizmo = create_transform_gizmo(&scene,ArrowType::Cone);

            scene.add(&cube);
            scene.add(&gizmo);
            rc_rcell(gizmo)
        };
        Self::add_events(view.clone(), scene.clone(), renderer.clone(), gizmo.clone());
        Self {
            view,
            scene,
            renderer,
            gizmo,
        }
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
    fn add_events(
        view: RcRcell<Viewport>,
        scene: RcRcell<Scene>,
        renderer: RcRcell<Renderer>,
        gizmo: RcRcell<Node>,
    ) {
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
        let a_view = view.clone();
        add_event(
            &document().get_element_by_id("toggle-perspective").unwrap(),
            "click",
            move |_| {
                handle_persp_toggle(a_view.clone());
            },
        );
        let a_view = view.clone();
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
            let a_scene = scene.clone();
            let a_rndr = renderer.clone();
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
                        create_primitive_node(&scene, &selected_prim)
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

        let a_rndr = renderer.clone();
        let renderer = renderer.borrow();
        let canvas = renderer.canvas();
        let a_view = view.clone();
        let a_gizmo = gizmo.clone();
        let a_scene = scene.clone();
        add_event(canvas, "click", move |e| {
                use ncollide3d::{shape::Cuboid, query::Ray, query::RayCast};
                use nalgebra::{Vector3, Point3, Isometry3};
                let target = Cuboid::new(Vector3::new(1.0,1.0,1.0));
                let scene = a_scene.borrow();
                let mut renderer = a_rndr.borrow_mut();
                let me = e.dyn_into::<MouseEvent>().unwrap();
                get_el("mesh-list").class_list().remove_1("shown").unwrap();

                let mut view = a_view.borrow_mut();
                let (hw, hh) = ((renderer.canvas().offset_width() / 2) as f32, (renderer.canvas().offset_height() / 2) as f32);
                let (x,y) = (me.offset_x() as f32 - hw, hh - me.offset_y() as f32);
                let ray_pos = view.screen_to_world([x/hw, y/hh, -1.0]);
                let ray_vec = view.screen_to_ray([x/hw, y/hh]);
                //log!("XY", x/hw, y/hh,"POINT", ray_pos, "RAY", ray_vec);
            let ray = Ray::new(ray_pos.into(), ray_vec.into());
            let mut mesh = a_gizmo.borrow().get_mesh().unwrap();
            if target.intersects_ray(&Isometry3::identity(), &ray) {
                mesh.material = Material::single_color_no_shade(1.0,0.,0.,1.);
                a_gizmo.borrow_mut().set_mesh(mesh);
            } else {
                mesh.material = Material::single_color_no_shade(0.8,0.8,0.8,1.);
                a_gizmo.borrow_mut().set_mesh(mesh);
            }
        });
        add_event(
            &document().get_element_by_id("add-mesh").unwrap(),
            "click",
            move |_| {
                get_el("mesh-list").class_list().toggle("shown").unwrap();
            },
        );
        let a_view = view.clone();
        add_event(&window(), "mousemove", move |e| {
            let me = e.dyn_into::<MouseEvent>().unwrap();
            let mut view = a_view.borrow_mut();
            let dy = if me.movement_y() > 0 { 0.1 } else { -0.1 };
            view.update_zoom(dy);
        });
        let a_view = view.clone();
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
        let a_view = view.clone();
        add_event(&window(), "keyup", move |e| {
            let keycode = e.dyn_into::<KeyboardEvent>().unwrap().code();
            if keycode == "KeyZ" {
                let mut view = a_view.borrow_mut();
                view.disable_zoom();
            }
        });
    }
}

