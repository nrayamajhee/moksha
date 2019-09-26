mod console;
pub use console::console_setup;

use crate::{
    dom_factory::{
        add_event, body, document, get_el, icon_btn_w_id, query_els, query_html_el, window,
    },
    mesh::{multiply, Geometry, Material},
    rc_rcell,
    renderer::DrawMode,
    scene::{
        primitives::{create_primitive_node, create_transform_gizmo, ArrowType, GizmoGrab},
        Gizmo, Node, Scene,
    },
    RcRcell, Renderer, Viewport,
};
use genmesh::generators::{IcoSphere, Plane};
use maud::{html, Markup};
use nalgebra::{Isometry3, Point3, Translation3, Unit, UnitQuaternion, Vector3};
use std::f32::consts::PI;
//use std::cell::RefCell;
//use std::rc::Rc;
use ncollide3d::{
    query::Ray,
    query::RayCast,
    shape::{Ball, ConvexHull, Cuboid, Plane as CollidePlane},
};
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlCanvasElement, HtmlElement, KeyboardEvent, MouseEvent, WheelEvent};

/// The main GUI editor that faciliates buttons to manipulate the scene, displays log in a separate
/// window, and displays the scene tree.
pub struct Editor {
    view: RcRcell<Viewport>,
    scene: RcRcell<Scene>,
    renderer: RcRcell<Renderer>,
    gizmo: RcRcell<Gizmo>,
    active_node: RcRcell<Option<RcRcell<Node>>>,
}

impl Editor {
    pub fn new(
        view: RcRcell<Viewport>,
        scene: RcRcell<Scene>,
        renderer: RcRcell<Renderer>,
    ) -> Self {
        let gizmo = {
            let mut scene = scene.borrow_mut();
            body()
                .insert_adjacent_html("beforeend", Self::markup(&scene).as_str())
                .expect("Couldn't insert console into the DOM!");
            let renderer = renderer.borrow();
            let grid = scene.object_from_mesh_and_name(
                Geometry::from_genmesh_no_normals(&Plane::subdivide(100, 100)),
                Material::single_color_no_shade(0.8, 0.8, 0.8, 1.0),
                "Grid",
            );
            grid.set_scale(50.0);
            grid.set_rotation(UnitQuaternion::from_euler_angles(PI / 2., 0., 0.));
            let grid = rc_rcell(grid);
            scene.add_with_mode(grid, DrawMode::Lines);
            let gizmo = create_transform_gizmo(&scene, ArrowType::Cone);
            scene.show(&gizmo, DrawMode::Triangle_no_depth);
            gizmo
        };
        let gizmo = rc_rcell(Gizmo::Translate(
            gizmo,
            GizmoGrab::None,
            Isometry3::identity(),
        ));
        let active_node = rc_rcell(None);
        let mut editor = Self {
            view,
            scene,
            renderer,
            gizmo,
            active_node,
        };
        editor.add_events();
        editor
    }
    pub fn set_active_node(&mut self, node: RcRcell<Node>) {
        let mut a_n = self.active_node.borrow_mut();
        *a_n = Some(node);
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
                            (Self::recurse_tree(&child.borrow()))
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
                    @let storage = scene.storage();
                    (Self::recurse_tree(scene.root()))
                }
            }
        };
        markup.into_string()
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
    fn ray_collides_w_node(ray: &Ray<f32>, node: &Node) -> Option<Isometry3<f32>> {
        let mut c_t = node.transform();
        let p_t = node.parent_tranform();
        let s = multiply(c_t.scale, p_t.scale);
        let verts: Vec<Point3<f32>> = node
            .mesh()
            .unwrap()
            .geometry
            .vertices
            .chunks(3)
            .map(|c| Point3::new(c[0] * s.x, c[1] * s.y, c[2] * s.z))
            .collect();
        if let Some(target) = ConvexHull::try_from_points(&verts) {
            let transform = (p_t * c_t).isometry;
            if target.intersects_ray(&transform, &ray) {
                Some(transform)
            } else {
                None
            }
        } else {
            None
        }
    }
    fn change_color(
        node: &Node,
        color: [f32; 3],
    ) {
        let mut mesh = node.mesh().unwrap();
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
                        rc_rcell(create_primitive_node(&scene, selected_prim.as_str().into()))
                    };
                    {
                        let mut scene = a_scene.borrow_mut();
                        scene.add(node);
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
        let a_node = self.active_node.clone();
        add_event(self.renderer.borrow().canvas(), "mousedown", move |e| {
            let mut renderer = a_rndr.borrow_mut();
            let mut view = a_view.borrow_mut();
            let mut gizmo = a_gizmo.borrow_mut();
            let scene = a_scene.borrow();

            get_el("mesh-list").class_list().remove_1("shown").unwrap();
            let me = e.dyn_into::<MouseEvent>().unwrap();
            let ray = Self::get_ray_from_screen(&me, &view, renderer.canvas());
            let (gizmo_node, gizmo_state, g_transform) = gizmo.inner_mut();

            let target = Ball::new(0.5);
            // if the central white ball is clicked
            if target.intersects_ray(&gizmo_node.transform().isometry, &ray) {
                let transform = Isometry3::from_parts(
                    Translation3::new(0., 0., 0.),
                    view.transform().inverse().rotation,
                );
                *gizmo_state = GizmoGrab::ViewPlane;
                *g_transform = transform;
                Self::change_color(gizmo_node, [1.,1.,1.]);
            } else {
                for child in gizmo_node.owned_children() {
                    let g_c = child.owned_children();
                    // if the arrows are clicked 
                    if g_c.len() > 0 {
                            let (tip, stem) = (&g_c[1], &g_c[0]);
                            let mut collided = false;
                            if let Some(_) = Self::ray_collides_w_node(&ray, tip) {
                               collided = true 
                            } else if let Some(_) = Self::ray_collides_w_node(&ray, stem) {
                               collided = true 
                            }
                            let transform = gizmo_node.transform().isometry;
                            let (color, g_state) = match child.info().name.as_str() {
                                "x-axis" => ([1.,0.,0.,], GizmoGrab::XAxis),
                                "y-axis" => ([0.,1.,0.,], GizmoGrab::YAxis),
                                "z-axis" => ([0.,0.,1.,], GizmoGrab::ZAxis),
                                _=>([0.,0.,0.], GizmoGrab::None),
                            };
                            if collided {
                                *gizmo_state = g_state;
                                *g_transform = transform;
                                Self::change_color(stem, color);
                                Self::change_color(tip, color);
                                break;
                            }
                    // if the cuboids are clicked 
                    } else {
                        if let Some(transform) = Self::ray_collides_w_node(&ray, &child) {
                            let (color, g_state) = match child.info().name.as_str() {
                                "pan_x" => ([1.,0.,0.,], GizmoGrab::XPlane),
                                "pan_y" => ([0.,1.,0.,], GizmoGrab::YPlane),
                                "pan_z" => ([0.,0.,1.,], GizmoGrab::ZPlane),
                                _=>([0.,0.,0.], GizmoGrab::None),
                            };
                            *gizmo_state = g_state;
                            *g_transform = transform;
                            Self::change_color(&child, color);
                            break;
                        }
                    }
                }
                // if any other object is clicked 
                for child in scene.root().children() {
                    if let Some(transform) = Self::ray_collides_w_node(&ray, &child.borrow()) {
                        let mut active_node = a_node.borrow_mut();
                        let v = child.borrow().transform().isometry.translation.vector;
                        *active_node = Some(child.clone());
                        gizmo_node.set_position([v.x, v.y, v.z]);
                    }
                }
            }
        });

        let a_view = self.view.clone();
        let a_rndr = self.renderer.clone();
        let a_gizmo = self.gizmo.clone();
        let renderer = self.renderer.clone();
        let a_node = self.active_node.clone();
        add_event(self.renderer.borrow().canvas(), "mousemove", move |e| {
            let renderer = a_rndr.borrow();
            let canvas = renderer.canvas();
            let mut view = a_view.borrow_mut();
            let gizmo = a_gizmo.borrow();
            let active_node = a_node.borrow();

            let (gizmo_node, gizmo_state, transform) = gizmo.inner();
            if *gizmo_state == GizmoGrab::None {
                return;
            }
            view.disable_rotation();
            let me = e.dyn_into::<MouseEvent>().unwrap();
            let ray = Self::get_ray_from_screen(&me, &view, &canvas);
            let axis = match gizmo_state {
                GizmoGrab::YAxis | GizmoGrab::XPlane => Vector3::x_axis(),
                GizmoGrab::XAxis | GizmoGrab::ZAxis | GizmoGrab::YPlane => Vector3::y_axis(),
                _=> Vector3::z_axis(),
            };
            let pan_view = CollidePlane::new(axis);
            if let Some(i) = pan_view.toi_and_normal_with_ray(&transform, &ray, true) {
                let g_p = gizmo_node.position();
                let poi = ray.point_at(i.toi);
                let pos = match gizmo_state {
                    GizmoGrab::XAxis => {
                        [poi.x, g_p.y, g_p.z] 
                    } 
                    GizmoGrab::YAxis => {
                        [g_p.x, poi.y, g_p.z] 
                    } 
                    GizmoGrab::ZAxis => {
                        [g_p.x, g_p.y, poi.z] 
                    } _=> {
                        [poi.x, poi.y, poi.z]
                    }
                };
                gizmo_node.set_position(pos);
                if let Some(node) = active_node.as_ref() {
                    node.borrow().set_position(pos);
                }
            }
        });

        let a_gizmo = self.gizmo.clone();
        add_event(self.renderer.borrow().canvas(), "wheel", move |e| {
            //let gizmo = a_gizmo.borrow();
            //let gizmo = gizmo.inner().0;
            //let we = e.dyn_into::<WheelEvent>().unwrap();
            //let scale = gizmo.scale().x;
            //let ds = if we.delta_y() > 0. { scale + 0.1 } else { scale - 0.1 };
            //gizmo.set_scale(ds);
        });

        let a_gizmo = self.gizmo.clone();
        add_event(self.renderer.borrow().canvas(), "mouseup", move |e| {
            let mut gizmo = a_gizmo.borrow_mut();
            let (gizmo_node, gizmo_state, g_transform) = gizmo.inner_mut();
            if *gizmo_state == GizmoGrab::None {
                return;
            }
            let color = match *gizmo_state {
                GizmoGrab::ViewPlane => [0.8,0.8,0.8],
                GizmoGrab::XAxis | GizmoGrab::XPlane => [0.8,0.,0.],
                GizmoGrab::YAxis | GizmoGrab::YPlane => [0.,0.8,0.],
                _=> [0.,0.,0.8],
            };
            let mut nodes = vec![gizmo_node];
            if *gizmo_state != GizmoGrab::ViewPlane {
                for child in gizmo_node.owned_children() {
                    let g_c = child.owned_children();
                    let name = child.info().name;
                    if g_c.len() > 0 {
                        let n_n = vec![&g_c[0], &g_c[1]];
                        if *gizmo_state == GizmoGrab::XAxis && name == "x-axis" {
                            nodes = n_n;
                            break;
                        } else if *gizmo_state == GizmoGrab::YAxis && name == "y-axis" {
                            nodes = n_n;
                            break;
                        } else if *gizmo_state == GizmoGrab::ZAxis && name == "z-axis" {
                            nodes = n_n;
                            break;
                        }
                    } else {
                        if *gizmo_state == GizmoGrab::XPlane && name =="pan_x" {
                            nodes = vec![child];
                            break;
                        } else if *gizmo_state == GizmoGrab::YPlane && name =="pan_y" {
                            nodes = vec![child];
                            break;
                        } else if *gizmo_state == GizmoGrab::ZPlane && name =="pan_z" {
                            nodes = vec![child];
                            break;
                        }
                    }
                }
            };
            *gizmo_state = GizmoGrab::None;
            *g_transform = Isometry3::identity();
            for each in nodes {
                Self::change_color(
                    each,
                    color,
                );
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
