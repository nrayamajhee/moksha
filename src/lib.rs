#![feature(proc_macro_hygiene)]
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use cgmath::{
	perspective, Decomposed, Deg, InnerSpace, Matrix4, Quaternion, Rad, Rotation3, Vector3,
};
use wasm_bindgen::JsCast;

use maud::html;
pub mod factory;
pub mod renderer;

pub use renderer::{Config, Renderer};

use factory::{add_event, body, document, request_animation_frame, window};
use web_sys::{console, HtmlElement, KeyboardEvent, Node};

use std::cell::RefCell;
use std::rc::Rc;

macro_rules! log {
    ( $( $t:tt )* ) => {
		let document = document();
		let console_el = document.get_element_by_id("console");
		let msg: String = format!( $( $t )* ).into();
		match console_el {
			Some(el) => {
				if !el.class_list().contains("shown") {
					el.class_list().add_1("shown").unwrap();
				}
				console::log_1(&JsValue::from_str(&msg));
				let current = document.query_selector("#console p.current").unwrap();
				match current {
					Some(el) => {el.class_list().remove_1("current").unwrap();}
					_ => ()
				}
				el.clone().insert_adjacent_html("afterbegin", &format!("<p class='current'>{}</p>", msg)).unwrap();
			},
			None => {
				console::log_1(&JsValue::from("Couldn't find console element. Only displaying to the dev console!"));
				web_sys::console::log_1(&JsValue::from_str(&msg));
			}
		}
    }
}

pub fn toggle_console(show: bool) {
	let console_el = document().get_element_by_id("console");
	match console_el {
		Some(el) => {
			if show {
				el.class_list().add_1("shown").unwrap();
			} else {
				el.class_list().remove_1("shown").unwrap();
			}
		}
		None => {
			log!("Couldn't find console element!");
		}
	}
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
	#[cfg(feature = "console_error_panic_hook")]
	console_error_panic_hook::set_once();

	let dom = html! {
		canvas #gl-canvas {}
		section #console {button #close-console{"Close Console"}}
	};

	document().set_title("Webshell | Rayamajhee");
	body().set_inner_html(dom.into_string().as_str());

	let mut renderer = Renderer::new(Config {
		selector: "#gl-canvas",
	})
	.expect("Can't create a webgl renderer! Make sure your browser supports it!");
	renderer
		.bind_shaders(
			r#"
            attribute vec4 position;
            // attribute vec4 color;
            attribute vec2 texCoord;
			attribute vec3  normal;

            uniform mat4 model, view, proj, normalMatrix;

            varying lowp vec4 f_color;
            varying lowp vec2 f_texCoord;
			varying lowp vec3 lighting;

            void main() {
                gl_Position = proj * view * model * position;
                // f_color = color;
                f_texCoord = texCoord;

				highp vec3 ambientLight = vec3(0.1, 0.1, 0.1);
				highp vec3 directionalLightColor = vec3(1, 1, 1);
				highp vec3 directionalVector = normalize(vec3(5.0, 5.0, 0.0));

				highp vec4 transformedNormal = normalMatrix * vec4(normal, 1.0);

				highp float directional = max(dot(transformedNormal.xyz, directionalVector), 0.0);
				lighting = ambientLight + (directionalLightColor * directional);
            }
        "#,
			r#"
            // varying lowp vec4 f_color;
            varying lowp vec2 f_texCoord;
			varying lowp vec3 lighting;

			uniform sampler2D sampler;

            void main() {
				// gl_FragColor = color;
				highp vec4 texelColor = texture2D(sampler, f_texCoord);
				gl_FragColor =vec4(texelColor.rgb * lighting, texelColor.a);
            }
        "#,
		)
		.expect("Can't bind shaders!");
	let vertices = vec![
		-1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, -1.0, 1.0, 1.0, // Front face
		-1.0, -1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, -1.0, -1.0, // Back face
		-1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, // Top face
		-1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, -1.0, -1.0, 1.0, // Bottom face
		1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, -1.0, 1.0, // Right face
		-1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, -1.0, // Left face
	];
	let indices = vec![
		0, 1, 2, 0, 2, 3, // front
		4, 5, 6, 4, 6, 7, // back
		8, 9, 10, 8, 10, 11, // top
		12, 13, 14, 12, 14, 15, // bottom
		16, 17, 18, 16, 18, 19, // right
		20, 21, 22, 20, 22, 23, // left
	];
	let mut colors = Vec::new();
	let face_colors = vec![
		[1.0, 1.0, 1.0, 1.0], // Front face: white
		[1.0, 0.0, 0.0, 1.0], // Back face: red
		[0.0, 1.0, 0.0, 1.0], // Top face: green
		[0.0, 0.0, 1.0, 1.0], // Bottom face: blue
		[1.0, 1.0, 0.0, 1.0], // Right face: yellow
		[1.0, 0.0, 1.0, 1.0], // Left face: purple
	];
	for each in face_colors.iter() {
		for _ in 0..4 {
			for i in 0..4 {
				colors.push(each[i]);
			}
		}
	}
	let tex_coord = vec![
		// Front
		0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, // Back
		0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, // Top
		0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, // Bottom
		0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, // Right
		0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, // Left
		0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0,
	];
	let normals = vec![
		// Front
		0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, // Back
		0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, // Top
		0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, // Bottom
		0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, // Right
		1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, // Left
		-1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0,
	];
	let proj = perspective(Rad::from(Deg(60.)), renderer.aspect_ratio(), 0.1, 100.);
	let view = Matrix4::look_at(
		[0.0, 3.0, 3.0].into(),
		[0.0, 0.0, 0.0].into(),
		Vector3::unit_y(),
	);
	let mut r = 0.;
	let mut axis = Vector3::new(0.0, 1.0, 0.0);
	let mut transform: Decomposed<Vector3<f32>, Quaternion<f32>> = Decomposed {
		scale: 1.0,
		rot: Quaternion::from_axis_angle(axis, Rad::from(Deg(r))),
		disp: [0.0, 0.0, 0.0].into(),
	};
	let model = Matrix4::from(transform);
	renderer
		.prepare_for_render(
			&vertices,
			&indices,
			&normals,
			&colors,
			&tex_coord,
			proj,
			view,
			model,
			"/assets/img/box_tex.png",
		)
		.expect("Couldn't setup renderer");

	let a_rndr = Rc::new(RefCell::new(renderer));
	let b_rndr = a_rndr.clone();
	let f = Rc::new(RefCell::new(None));
	let g = f.clone();
	*g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
		let mut renderer = a_rndr.borrow_mut();
		r = (r + 1.) % 360.;
		axis.x = axis.x + 1.;
		axis.y = axis.y + 2.;
		transform.rot = Quaternion::from_axis_angle(axis.normalize(), Rad::from(Deg(r)));
		let model = Matrix4::from(transform);
		renderer
			.update_transform(model)
			.expect("Couldn't update transform");
		renderer.render().expect("Couldn't render!");
		request_animation_frame(f.borrow().as_ref().unwrap());
	}) as Box<dyn FnMut()>));
	request_animation_frame(g.borrow().as_ref().unwrap());
	let window = window();
	add_event(&window, "resize", move |_| {
		let mut renderer = b_rndr.borrow_mut();
		renderer.resize();
	});
	add_event(&document().get_element_by_id("close-console").unwrap(), "click", move |_|{
		toggle_console(false);
	});
	log!("Console enabled!");
	add_event(&window, "keydown", move |e| {
		if e.dyn_into::<KeyboardEvent>().unwrap().code() == "Backquote" {
			let console_el = document().get_element_by_id("console");
			match console_el {
				Some(el) => {
					let shown = el.class_list().contains("shown");
					toggle_console(!shown);
				}
				None => {
					log!("Didn't find console element. Not adding event handlers!");
				}
			}
		}
	});
	request_animation_frame(g.borrow().as_ref().unwrap());
	Ok(())
}
