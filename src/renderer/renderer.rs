use super::shader::{
    bind_buffer_and_attribute, bind_index_buffer, create_color_program, create_simple_program,
    create_texture_program, create_vertex_color_program, create_wire_program, set_bool, set_f32,
    set_i32, set_mat4, set_vec3, set_vec4, ShaderType,
};
use crate::dom_factory::{get_canvas, resize_canvas};
use crate::{controller::Viewport, log, mesh::Mesh, scene::Scene, LightType};
use std::collections::HashMap;
use wasm_bindgen::{prelude::*, JsCast};
use strum::IntoEnumIterator;
use web_sys::{
    HtmlCanvasElement, HtmlElement, WebGl2RenderingContext as GL, WebGlProgram,
    WebGlVertexArrayObject,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DrawMode {
    Points,
    Wireframe,
    Lines,
    PointyLines,
    Triangle,
    TriangleNoDepth,
}

#[derive(Debug)]
pub struct RenderConfig {
    pub selector: &'static str,
    pub pixel_ratio: f64,
}

/// WebGL renderer that compiles, binds and executes all shaders; also capable of handling window resizes and configuration changes
#[wasm_bindgen]
#[derive(Debug)]
pub struct Renderer {
    canvas: HtmlCanvasElement,
    ctx: GL,
    aspect_ratio: f32,
    shaders: HashMap<ShaderType, WebGlProgram>,
    config: RenderConfig,
}

impl Renderer {
    pub fn new(config: RenderConfig) -> Self {
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
            create_simple_program(&ctx).expect("Can't create simple shader!"),
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
        shaders.insert(
            ShaderType::Wireframe,
            create_wire_program(&ctx).expect("can't create texture shader!"),
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
            // bind vertices
            if shader_type == ShaderType::Wireframe {
                let mut vertices = Vec::new();
                let mut bary_buffer = Vec::new();
                let barycentric: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
                for each in mesh.geometry.indices.iter() {
                    let i = (each * 3) as usize;
                    vertices.push(mesh.geometry.vertices[i]);
                    vertices.push(mesh.geometry.vertices[i + 1]);
                    vertices.push(mesh.geometry.vertices[i + 2]);
                }
                for _ in 0..vertices.len() / 9 {
                    for each in &barycentric {
                        bary_buffer.push(*each);
                    }
                }
                bind_buffer_and_attribute(&self.ctx, &program, "position", &vertices, 3)
                    .expect("Can't bind postion");
                bind_buffer_and_attribute(&self.ctx, &program, "barycentric", &bary_buffer[..], 3)
                    .expect("Can't bind postion");
            } else {
                bind_buffer_and_attribute(
                    &self.ctx,
                    &program,
                    "position",
                    &mesh.geometry.vertices,
                    3,
                )
                .expect("Can't bind postion");
            }
            // bind normals
            if shader_type != ShaderType::Simple && shader_type != ShaderType::Wireframe {
                bind_buffer_and_attribute(&self.ctx, &program, "normal", &mesh.geometry.normals, 3)
                    .expect("Can't bind normals");
            // bind vertex color
            } else if shader_type == ShaderType::VertexColor {
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
            // bind texture
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
        let gl = &self.ctx;
        gl.clear_color(0.1, 0.1, 0.1, 1.0);
        gl.clear_depth(1.0);
        gl.depth_func(GL::LEQUAL);
        gl.front_face(GL::CCW);
        gl.cull_face(GL::BACK);
        gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
        log!("Renderer is ready to draw");
    }
    pub fn render(&self, scene: &Scene, viewport: &Viewport) {
        let gl = &self.ctx;
        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);
        let storage = scene.storage();
        let storage = storage.borrow();
        // bind lights per frame for only one program
        {
            let program = self.shaders.get(&ShaderType::Color).unwrap();
            gl.use_program(Some(&program));
            let mut num_l_amb = 0;
            let mut num_l_point = 0;
            for light in storage.lights() {
                if !light.light {
                    continue
                }
                match light.light_type {
                    LightType::Ambient => {
                        set_vec3(gl, program, &format!("amb_lights[{}].color",num_l_amb), &light.color);
                        num_l_amb += 1;
                    }
                    LightType::Point => {
                        let position = (storage.parent_tranform(light.node_id)
                            * storage.transform(light.node_id))
                        .isometry
                        .translation
                        .vector
                        .data;
                        let range = 100.;
                        let linear = 4.5 / range;
                        let quadratic = 7.5 / (range * range);
                        set_f32(gl, program, &format!("point_lights[{}].linear",num_l_point), linear);
                        set_f32(gl, program, &format!("point_lights[{}].quadratic",num_l_point), quadratic);
                        set_vec3(gl, program, &format!("point_lights[{}].position",num_l_point), &position);
                        set_vec3(gl, program, &format!("point_lights[{}].color",num_l_point), &light.color);
                        num_l_point += 1;
                    }
                    _=>()
                }
            }
            set_i32(gl, program, "num_l_amb", num_l_amb as i32);
            set_i32(gl, program, "num_l_point", num_l_point as i32);
            set_vec3(gl, program, "eye", &viewport.eye());
        }
        // bind view and projection per frame per program
        {
            for each in ShaderType::iter() {
                let program = self.shaders.get(&each).unwrap();
                gl.use_program(Some(&program));
                set_mat4(gl, &program, "view", &viewport.view());
                set_mat4(gl, &program, "proj", &viewport.proj());
            }
        }
        // bind each mesh per frame
        let meshes = storage.meshes();
        for (i, mesh) in meshes.iter().enumerate() {
            if let Some(mesh) = mesh {
                let info = storage.info(i);
                if !info.render {
                    continue;
                }
                let vao = storage.vao(i);
                let shader_type = mesh.material.shader_type;
                let program = self.shaders.get(&shader_type).unwrap();
                gl.use_program(Some(&program));
                gl.bind_vertex_array(vao);
                if shader_type == ShaderType::Simple
                    || shader_type == ShaderType::Color
                    || shader_type == ShaderType::Wireframe
                {
                    set_vec4(
                        gl,
                        program,
                        "color",
                        &mesh
                            .material
                            .color
                            .expect("Can't render a color materaial without a color!"),
                    );
                }
                if shader_type == ShaderType::Color {
                    set_bool(gl, program, "flat_shade", mesh.material.flat_shade);
                }
                if shader_type == ShaderType::Texture {
                    gl.active_texture(GL::TEXTURE0);
                    set_i32(gl, program, "sampler", 0);
                }
                let model = storage.parent_tranform(i) * storage.transform(i);
                set_mat4(gl, program, "model", &model.to_homogeneous());
                let indices = &mesh.geometry.indices;
                bind_index_buffer(gl, &indices).expect("Can't bind index buffer!");

                gl.disable(GL::BLEND);
                gl.depth_mask(true);
                gl.enable(GL::CULL_FACE);
                gl.enable(GL::DEPTH_TEST);
                match info.draw_mode {
                    DrawMode::Wireframe => {
                        gl.depth_mask(false);
                        gl.enable(GL::BLEND);
                        gl.disable(GL::CULL_FACE);
                        gl.draw_arrays(GL::TRIANGLES, 0, indices.len() as i32);
                    }
                    DrawMode::Triangle => {
                        gl.draw_elements_with_i32(
                            GL::TRIANGLES,
                            indices.len() as i32,
                            GL::UNSIGNED_SHORT,
                            0,
                        );
                    }
                    DrawMode::TriangleNoDepth => {
                        gl.disable(GL::DEPTH_TEST);
                        gl.draw_elements_with_i32(
                            GL::TRIANGLES,
                            indices.len() as i32,
                            GL::UNSIGNED_SHORT,
                            0,
                        );
                    }
                    DrawMode::Lines => {
                        gl.draw_elements_with_i32(
                            GL::LINES,
                            indices.len() as i32,
                            GL::UNSIGNED_SHORT,
                            0,
                        );
                    }
                    _ => (),
                }
            }
        }
        gl.bind_vertex_array(None);
        gl.bind_buffer(GL::ARRAY_BUFFER, None);
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, None);
        gl.use_program(None);
    }
    pub fn resize(&mut self) {
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
