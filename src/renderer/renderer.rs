use crate::dom_factory::{add_event, document, window};
use cgmath::{prelude::*, Matrix4, SquareMatrix};
use js_sys::{Float32Array, Uint16Array, Uint32Array, Uint8Array};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    HtmlCanvasElement, HtmlImageElement, WebGlBuffer, WebGlProgram, WebGlRenderingContext as GL,
    WebGlShader,
};

use super::{
    shader::{
        bind_attribute, bind_buffer_f32, bind_index_buffer, bind_matrix, bind_uniform_1i,
        bind_vector, color_program, texture_program, vertex_color_program,
        ShaderType,
    },
    Geometry, Material, Viewport,
};
use crate::dom_factory::{get_canvas, resize_canvas};

pub struct Config {
    pub selector: &'static str,
    pub pixel_ratio: f64,
}

pub struct Renderer {
    canvas: HtmlCanvasElement,
    ctx: GL,
    aspect_ratio: f32,
    shaders: Vec<WebGlProgram>,
    active_shader: usize,
    num_elements: usize,
    config: Config,
}

impl Renderer {
    pub fn new(config: Config) -> Result<Self, JsValue> {
        let mut canvas = get_canvas(config.selector);
        let aspect_ratio = resize_canvas(&mut canvas, config.pixel_ratio);
        let ctx = canvas.get_context("webgl")?.unwrap().dyn_into::<GL>()?;

        let mut shaders = Vec::new();
        shaders.push(color_program(&ctx)?);
        shaders.push(vertex_color_program(&ctx)?);
        shaders.push(texture_program(&ctx)?);
        Ok(Self {
            canvas,
            ctx,
            aspect_ratio,
            shaders,
            num_elements: 0,
            active_shader: 0,
            config,
        })
    }
    pub fn resize(&mut self) {
        self.aspect_ratio = resize_canvas(&mut self.canvas, self.config.pixel_ratio);
        use cgmath::{perspective, Deg, Rad};
        let proj = perspective(Rad::from(Deg(60.)), self.aspect_ratio(), 0.1, 100.);
        self.ctx.viewport(
            0,
            0,
            self.canvas.width() as i32,
            self.canvas.height() as i32,
        );
        let program = &(self.shaders[self.active_shader]);
        bind_matrix(&self.ctx, program, "proj", &proj);
    }
    pub fn bind_geometry(&mut self, geometry: &Geometry) -> Result<(), JsValue> {
        self.num_elements = geometry.indices.len();
        bind_buffer_f32(&self.ctx, &geometry.vertices)?;
        bind_buffer_f32(&self.ctx, &geometry.normals)?;
        bind_index_buffer(&self.ctx, &geometry.indices[..])?;
        Ok(())
    }
    pub fn bind_material(&mut self, material: &Material) -> Result<(), JsValue> {
        self.active_shader = match material.shader_type {
            ShaderType::Color => 0,
            ShaderType::VertexColor => 1,
            ShaderType::Texture => 2,
        };
        let program = &(self.shaders[self.active_shader]);
        self.ctx.use_program(Some(program));
        bind_attribute(&self.ctx, program, "position", 3);
        bind_attribute(&self.ctx, program, "normal", 3);
        match material.shader_type {
            ShaderType::Color => {
                match &material.color {
                    Some(color) => {
                        bind_vector(&self.ctx, program, "color", color);
                    },
                    None => {return Err(JsValue::from("Can't render a color materaial without a color!"));}
                }
            }
            ShaderType::VertexColor => {
                match &material.vertex_colors {
                    Some(vertex_color) => {
                        bind_buffer_f32(&self.ctx, &vertex_color)?;
                        bind_attribute(&self.ctx, program, "color", 4);
                    },
                    None => {return Err(JsValue::from("Can't render a vertex color materaial without a color buffer!"));}
                }
            }
            ShaderType::Texture => {
                match &material.tex_coords {
                    Some(tex_coords) => {
                        bind_buffer_f32(&self.ctx, &tex_coords)?;
                        bind_attribute(&self.ctx, program, "texCoord", 2);
                        self.ctx.active_texture(GL::TEXTURE0);
                        bind_uniform_1i(&self.ctx, program, "sampler", 0);
                    },
                    None => {return Err(JsValue::from("Can't setup texture coordinates!"));}
                }
            }
        };
        Ok(())
    }
    pub fn update_transform(&mut self, model: &Matrix4<f32>) {
       // update nomals and view matrix per frame
        let program = &(self.shaders[self.active_shader]);
        bind_matrix(&self.ctx, program, "model", model);
        let normal_matrix = model.invert().unwrap().transpose();
        bind_matrix(&self.ctx, program, "normalMatrix", &normal_matrix);
    }
    pub fn update_viewport(&mut self, viewport: &Viewport) {
        let program = &(self.shaders[self.active_shader]);
        bind_matrix(&self.ctx, program, "view", &viewport.view);
        bind_matrix(&self.ctx, program, "proj", &viewport.proj);
    }
    pub fn prepare_renderer(&mut self) {
        self.ctx.clear_color(0.0, 0.0, 0.0, 1.0);
        self.ctx.clear_depth(1.0);
        self.ctx.depth_func(GL::LEQUAL);
        self.ctx.enable(GL::DEPTH_TEST);
        self.ctx.front_face(GL::CCW);
        self.ctx.cull_face(GL::BACK);
        self.ctx.enable(GL::CULL_FACE);
    }
    pub fn render(&self) -> Result<(), JsValue> {
        self.ctx.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        self.ctx.draw_elements_with_i32(
            GL::TRIANGLES,
            self.num_elements as i32,
            GL::UNSIGNED_SHORT,
            0,
        );
        Ok(())
    }
    pub fn get_context(&self) -> &GL {
        &self.ctx
    }
    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }
}
