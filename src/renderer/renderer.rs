use super::shader::{
    bind_buffer_and_attribute, bind_index_buffer, bind_uniform_i32, bind_uniform_mat4,
    bind_uniform_vec4, create_color_program, create_simple_program, create_texture_program,
    create_vertex_color_program, ShaderType,
};
use crate::dom_factory::{get_canvas, resize_canvas};
use crate::mesh::{Geometry, Material};
use crate::{
    controller::Viewport,
    log,
    scene::{Node, Scene},
    mesh::Mesh,
};
use genmesh::generators::Plane;
use nalgebra::UnitQuaternion;
use std::collections::HashMap;
use std::f32::consts::PI;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{
    HtmlCanvasElement, HtmlElement, WebGl2RenderingContext as GL, WebGlProgram,
    WebGlVertexArrayObject,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DrawMode {
    Points,
    Lines,
    PointyLines,
    Triangle,
    None,
}

pub struct Config {
    pub selector: &'static str,
    pub pixel_ratio: f64,
}

#[wasm_bindgen]
pub struct Renderer {
    canvas: HtmlCanvasElement,
    ctx: GL,
    aspect_ratio: f32,
    shaders: HashMap<ShaderType, WebGlProgram>,
    config: Config,
}

impl Renderer {
    pub fn new(config: Config) -> Self {
        let mut canvas = get_canvas(config.selector);
        let aspect_ratio = resize_canvas(&mut canvas, config.pixel_ratio);
        let ctx = canvas
            .get_context("webgl2")
            .expect("Can't create webgl2 context. Make sure your browser supports WebGL2")
            .unwrap()
            .dyn_into::<GL>()
            .unwrap();

        let mut shaders = HashMap::new();
        shaders.insert(
            ShaderType::Simple,
            create_simple_program(&ctx).expect("Can't create color shader!"),
        );
        shaders.insert(
            ShaderType::Color,
            create_color_program(&ctx).expect("Can't create color shader!"),
        );
        shaders.insert(
            ShaderType::VertexColor,
            create_vertex_color_program(&ctx).expect("Can't create vertex color shader!"),
        );
        shaders.insert(
            ShaderType::Texture,
            create_texture_program(&ctx).expect("can't create texture shader!"),
        );
        log!("Renderer created");
        Self {
            canvas,
            ctx,
            aspect_ratio,
            shaders,
            config,
        }
    }
    pub fn create_vao(&self, mesh: &Option<Mesh>) -> Option<WebGlVertexArrayObject> {
        if let Some(mesh) = mesh {
            let shader_type = mesh.material.shader_type;
            let program = self
                .shaders
                .get(&shader_type)
                .expect("Can't find the program!");
            let vao = self.ctx.create_vertex_array().expect("Can't creat VAO");
            self.ctx.bind_vertex_array(Some(&vao));
            bind_buffer_and_attribute(
                &self.ctx,
                &program,
                "position",
                &mesh.geometry.vertices,
                3,
            )
            .expect("Can't bind postion");
            if shader_type != ShaderType::Simple {
                bind_buffer_and_attribute(
                    &self.ctx,
                    &program,
                    "normal",
                    &mesh.geometry.normals,
                    3,
                )
                .expect("Can't bind normals");
            }
            if shader_type == ShaderType::VertexColor {
                bind_buffer_and_attribute(
                    &self.ctx,
                    &program,
                    "color",
                    mesh.material
                        .vertex_colors
                        .as_ref()
                        .expect("Expected vertex color, found nothing!"),
                    4,
                )
                .expect("Couldn't bind vertex colors.");
            } else if shader_type == ShaderType::Texture {
                bind_buffer_and_attribute(
                    &self.ctx,
                    &program,
                    "texCoord",
                    mesh.material
                        .tex_coords
                        .as_ref()
                        .expect("Expected texture coordinates, found nothing!"),
                    2,
                )
                .expect("Couldn't bind tex coordinates");
            }
            self.ctx.bind_buffer(GL::ARRAY_BUFFER, None);
            self.ctx.bind_vertex_array(None);
            Some(vao)
        } else {
            None
        }
    }
    pub fn setup_renderer(&self) {
        self.ctx.clear_color(0.1, 0.1, 0.1, 1.0);
        self.ctx.clear_depth(1.0);
        self.ctx.depth_func(GL::LEQUAL);
        self.ctx.enable(GL::DEPTH_TEST);
        self.ctx.front_face(GL::CCW);
        self.ctx.cull_face(GL::BACK);
        self.ctx.enable(GL::CULL_FACE);
        log!("Renderer is setup!");
    }
    pub fn render(&self, scene: &Scene) {
        self.ctx.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);
        let storage = scene.get_storage();
        let storage = storage.borrow();
        let meshes = storage.meshes();
        for (i, mesh) in meshes.iter().enumerate() {
            if let Some(mesh) = mesh {
                let draw_mode = storage.get_info(i).draw_mode;
                let vao = storage.get_vao(i);
                if draw_mode == DrawMode::None {
                    continue;
                }
                let shader_type = mesh.material.shader_type;
                let program = self.shaders.get(&shader_type).unwrap();
                self.ctx.use_program(Some(&program));
                self.ctx.bind_vertex_array(vao);
                if shader_type == ShaderType::Color || shader_type == ShaderType::Simple {
                    bind_uniform_vec4(
                        &self.ctx,
                        program,
                        "color",
                        &mesh
                            .material
                            .color
                            .expect("Can't render a color materaial without a color!"),
                    );
                } else if shader_type == ShaderType::Texture {
                    self.ctx.active_texture(GL::TEXTURE0);
                    bind_uniform_i32(&self.ctx, program, "sampler", 0);
                }
                let transform = storage.get_transform(i).to_homogeneous();
                let p_transform = storage.get_parent_transform(i).to_homogeneous();
                let model = p_transform * transform;
                bind_uniform_mat4(&self.ctx, program, "model", &model);
                let normal_matrix = transform.transpose();
                if shader_type != ShaderType::Simple {
                    bind_uniform_mat4(&self.ctx, program, "normalMatrix", &normal_matrix);
                }
                let indices = &mesh.geometry.indices;
                bind_index_buffer(&self.ctx, &indices).expect("Can't bind index buffer!");
                match draw_mode {
                    DrawMode::Triangle => {
                        let name = storage.get_info(i).name;
                        if name == "translation" {
                            self.ctx.disable(GL::DEPTH_TEST);
                        }
                        self.ctx.draw_elements_with_i32(
                            GL::TRIANGLES,
                            indices.len() as i32,
                            GL::UNSIGNED_SHORT,
                            0,
                        );
                        if name == "translation" {
                            self.ctx.enable(GL::DEPTH_TEST);
                        }
                    }
                    DrawMode::Lines => {
                        self.ctx.draw_elements_with_i32(
                            GL::LINES,
                            indices.len() as i32,
                            GL::UNSIGNED_SHORT,
                            0,
                        );
                    }
                    _ => (),
                }
                self.ctx.bind_vertex_array(None);
                self.ctx.bind_buffer(GL::ARRAY_BUFFER, None);
                self.ctx.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, None);
                self.ctx.use_program(None);
            }
        }
    }
    pub fn update_viewport(&self, viewport: &Viewport) {
        for (_, program) in &self.shaders {
            self.ctx.use_program(Some(&program));
            bind_uniform_mat4(&self.ctx, &program, "view", &viewport.view());
            bind_uniform_mat4(&self.ctx, &program, "proj", &viewport.proj());
            self.ctx.use_program(None);
        }
    }
    pub fn resize(&mut self, viewport: &mut Viewport) {
        log!("Renderer resized");
        self.aspect_ratio = resize_canvas(&mut self.canvas, self.config.pixel_ratio);
        // log!("New aspect ratio: {:?}", self.aspect_ratio());
        self.ctx.viewport(
            0,
            0,
            self.canvas.width() as i32,
            self.canvas.height() as i32,
        );
    }
    pub fn context(&self) -> &GL {
        &self.ctx
    }
    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }
    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }
    pub fn width(&self) -> u32 {
        self.canvas.width()
    }
    pub fn height(&self) -> u32 {
        self.canvas.height()
    }
    pub fn change_cursor(&self, cursory_type: CursorType) {
        let canvas_style = self
            .canvas
            .clone()
            .dyn_into::<HtmlElement>()
            .unwrap()
            .style();
        match cursory_type {
            CursorType::Pointer => {
                canvas_style.set_property("cursor", "default").unwrap();
            }
            CursorType::Grab => {
                canvas_style.set_property("cursor", "grabbing").unwrap();
            }
            CursorType::ZoomIn => {
                canvas_style.set_property("cursor", "zoom-in").unwrap();
            }
            CursorType::ZoomOut => {
                canvas_style.set_property("cursor", "zoom-out").unwrap();
            }
        }
    }
}

pub enum CursorType {
    Pointer,
    Grab,
    ZoomIn,
    ZoomOut,
}
