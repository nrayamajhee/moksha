use crate::factory::{add_event, document, window};
use cgmath::{prelude::*, Matrix4, SquareMatrix};
use js_sys::{Float32Array, Uint16Array, Uint8Array};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    HtmlCanvasElement, HtmlImageElement, WebGlBuffer, WebGlProgram, WebGlRenderingContext as GL,
    WebGlShader,
};

pub struct Config {
    pub selector: &'static str,
}

pub struct BufferObject {
    pub vertices: WebGlBuffer,
    pub indices: WebGlBuffer,
    pub normals: WebGlBuffer,
    pub tex_coords: WebGlBuffer,
    pub colors: WebGlBuffer,
}

pub struct UniformObject {
    pub proj: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub model: Matrix4<f32>,
}

fn is_power_of_2(val: u32) -> bool {
    return (val & (val - 1)) == 0;
}

pub struct Renderer {
    canvas: HtmlCanvasElement,
    ctx: GL,
    aspect_ratio: f32,
    programs: Vec<WebGlProgram>,
    num_triangles: usize,
    buffers: Option<BufferObject>,
    uniforms: Option<UniformObject>,
}

impl Renderer {
    pub fn new(config: Config) -> Result<Self, JsValue> {
        let mut canvas = Self::get_canvas(config.selector);
        let aspect_ratio = Self::resize_canvas(&mut canvas);
        let ctx = canvas.get_context("webgl")?.unwrap().dyn_into::<GL>()?;

        let programs = Vec::new();
        Ok(Self {
            canvas,
            ctx,
            aspect_ratio,
            programs,
            num_triangles: 0,
            buffers: None,
            uniforms: None,
        })
    }
    fn get_canvas(selector: &str) -> HtmlCanvasElement {
        let canvas = document().query_selector(selector).unwrap().expect(
            format!(
                "Couldn't find canvas with selector `{}` ! Make sure your DOM has a canvas element",
                selector
            )
            .as_str(),
        );
        let canvas: HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Can't convert the dom element to HtmlCanvasElement!");
        canvas
    }
    pub fn resize_canvas(canvas: &mut HtmlCanvasElement) -> f32 {
        let window = window();
        let pixel_ratio = window.device_pixel_ratio() / 2.;
        let width: u32 = (window.inner_width().unwrap().as_f64().unwrap() * pixel_ratio) as u32;
        let height: u32 = (window.inner_height().unwrap().as_f64().unwrap() * pixel_ratio) as u32;
        canvas.set_width(width);
        canvas.set_height(height);
        width as f32 / height as f32
    }
    pub fn bind_shaders(&mut self, vertext: &str, fragment: &str) -> Result<(), String> {
        let vert_shader = self.compile_shader(GL::VERTEX_SHADER, vertext)?;
        let frag_shader = self.compile_shader(GL::FRAGMENT_SHADER, fragment)?;
        let program = self.link_program(&vert_shader, &frag_shader)?;
        self.ctx.validate_program(&program);
        if (self
            .ctx
            .get_program_parameter(&program, GL::VALIDATE_STATUS))
        .as_bool()
        .unwrap_or(false)
        {
            self.programs.push(program);
            Ok(())
        } else {
            Err(self
                .ctx
                .get_program_info_log(&program)
                .unwrap_or_else(|| String::from("Unknown error creating program object")))
        }
    }
    fn compile_shader(&self, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
        let shader = self
            .ctx
            .create_shader(shader_type)
            .ok_or_else(|| String::from("Unable to create shader object"))?;
        self.ctx.shader_source(&shader, source);
        self.ctx.compile_shader(&shader);

        if self
            .ctx
            .get_shader_parameter(&shader, GL::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(shader)
        } else {
            Err(self
                .ctx
                .get_shader_info_log(&shader)
                .unwrap_or_else(|| String::from("Unknown error creating shader")))
        }
    }
    fn link_program(
        &self,
        vert_shader: &WebGlShader,
        frag_shader: &WebGlShader,
    ) -> Result<WebGlProgram, String> {
        let program = self
            .ctx
            .create_program()
            .ok_or_else(|| String::from("Unable to create shader object"))?;

        self.ctx.attach_shader(&program, vert_shader);
        self.ctx.attach_shader(&program, frag_shader);
        self.ctx.link_program(&program);

        if self
            .ctx
            .get_program_parameter(&program, GL::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(program)
        } else {
            Err(self
                .ctx
                .get_program_info_log(&program)
                .unwrap_or_else(|| String::from("Unknown error creating program object")))
        }
    }
    pub fn prepare_for_render(
        &mut self,
        vertices: &Vec<f32>,
        indices: &Vec<u16>,
        normals: &Vec<f32>,
        colors: &Vec<f32>,
        tex_coord: &Vec<f32>,
        proj: Matrix4<f32>,
        view: Matrix4<f32>,
        model: Matrix4<f32>,
        img_url: &str,
    ) -> Result<(), JsValue> {
        self.num_triangles = indices.len();
        let vertices = self.bind_buffer_f32(vertices)?;
        self.bind_attribute("position", 3)?;
        let indices = self.bind_index_buffer(&indices[..])?;
        let normals = self.bind_buffer_f32(normals)?;
        self.bind_attribute("normal", 3)?;
        // let colors = self.bind_buffer(colors, BufferType::F32)?;
        // self.bind_attribute("color", 4)?;
        let colors = self.ctx.create_buffer().unwrap();
        let tex_coords = self.bind_buffer_f32(tex_coord)?;
        self.bind_attribute("texCoord", 2)?;

        self.buffers = Some(BufferObject {
            vertices,
            indices,
            normals,
            tex_coords,
            colors,
        });

        self.uniforms = Some(UniformObject { proj, view, model });
        self.ctx.use_program(Some(&self.programs[0]));
        self.bind_uniforms()?;

        self.load_texture(img_url)?;
        self.ctx.active_texture(GL::TEXTURE0);
        self.bind_uniform_1i("sampler", 0)?;

        self.ctx.clear_color(0.0, 0.0, 0.0, 1.0);
        self.ctx.clear_depth(1.0);
        self.ctx.depth_func(GL::LEQUAL);
        self.ctx.enable(GL::DEPTH_TEST);
        self.ctx.front_face(GL::CCW);
        self.ctx.cull_face(GL::BACK);
        self.ctx.enable(GL::CULL_FACE);

        Ok(())
    }
    fn bind_uniform_1i(&self, name: &str, value: i32) -> Result<(), String> {
        if self.programs.len() == 0 {
            Err(String::from("No program found to bind the unform"))
        } else {
            let program = &self.programs[0];
            let attrib = self
                .ctx
                .get_uniform_location(program, name)
                .expect(format!("Can't bind uniform: {}", name).as_str());
            self.ctx.uniform1i(Some(&attrib), value);
            Ok(())
        }
    }
    fn bind_buffer_f32(&mut self, data: &[f32]) -> Result<WebGlBuffer, JsValue> {
        let buffer = self.ctx.create_buffer().ok_or("failed to create buffer")?;
        self.ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));
        let buffer_array = unsafe { Float32Array::view(&data) };
        self.ctx.buffer_data_with_array_buffer_view(
            GL::ARRAY_BUFFER,
            &buffer_array,
            GL::STATIC_DRAW,
        );
        Ok(buffer)
    }
    fn bind_index_buffer(&mut self, data: &[u16]) -> Result<WebGlBuffer, JsValue> {
        let buffer = self.ctx.create_buffer().ok_or("failed to create buffer")?;
        self.ctx
            .bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&buffer));
        let buffer_array = unsafe { Uint16Array::view(&data) };
        self.ctx.buffer_data_with_array_buffer_view(
            GL::ELEMENT_ARRAY_BUFFER,
            &buffer_array,
            GL::STATIC_DRAW,
        );

        Ok(buffer)
    }
    fn bind_attribute(&self, name: &str, size: i32) -> Result<(), String> {
        if self.programs.len() > 0 {
            let attribute = self.ctx.get_attrib_location(&self.programs[0], name);
            self.ctx
                .vertex_attrib_pointer_with_i32(attribute as u32, size, GL::FLOAT, false, 0, 0);
            self.ctx.enable_vertex_attrib_array(attribute as u32);
            Ok(())
        } else {
            Err(String::from("No shader programs available"))
        }
    }
    fn bind_uniforms(&self) -> Result<(), String> {
        if self.programs.len() <= 0 {
            Err(String::from("No shader programs available"))
        } else {
            self.bind_matrix("model", &self.uniforms.as_ref().unwrap().model);
            self.bind_matrix("view", &self.uniforms.as_ref().unwrap().view);
            self.bind_matrix("proj", &self.uniforms.as_ref().unwrap().proj);
            Ok(())
        }
    }
    pub fn bind_matrix(&self, attribute: &str, matrix: &Matrix4<f32>) {
        let program = &self.programs[0];
        let mat: &[f32; 16] = matrix.as_ref();
        let mat_attrib = self
            .ctx
            .get_uniform_location(program, attribute)
            .expect(format!("Can't bind uniform: {}", attribute).as_str());
        self.ctx
            .uniform_matrix4fv_with_f32_array(Some(&mat_attrib), false, mat);
    }
    fn load_texture(&self, url: &str) -> Result<(), JsValue> {
        let texture = self.ctx.create_texture().expect("Can't create texture!");
        self.ctx.bind_texture(GL::TEXTURE_2D, Some(&texture));
        let pixel = unsafe { Uint8Array::view(&[255, 0, 255, 255]) };
        self.ctx
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
                GL::TEXTURE_2D,
                0,
                GL::RGBA as i32,
                1,
                1,
                0,
                GL::RGBA,
                GL::UNSIGNED_BYTE,
                Some(&pixel),
            )?;
        let image = HtmlImageElement::new().expect("Can't create Image Element");
        let img = Rc::new(RefCell::new(image));
        let a_img = img.clone();
        // couldn't avoid this
        let gl = self.ctx.clone();
        add_event(&img.borrow(), "load", move |_| {
            let image = a_img.borrow();
            gl.bind_texture(GL::TEXTURE_2D, Some(&texture));
            gl.tex_image_2d_with_u32_and_u32_and_image(
                GL::TEXTURE_2D,
                0,
                GL::RGBA as i32,
                GL::RGBA,
                GL::UNSIGNED_BYTE,
                &image,
            )
            .expect("Couldn't bind image as texture!");
            if is_power_of_2(image.width()) && is_power_of_2(image.height()) {
                gl.generate_mipmap(GL::TEXTURE_2D);
            } else {
                gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
                gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
                gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32);
                gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);
            }
        });
        img.borrow_mut().set_src(url);
        Ok(())
    }
    pub fn update_transform(&mut self, model: Matrix4<f32>) -> Result<(), String> {
        // update nomals and view matrix per frame
        match &mut self.uniforms {
            Some(uniforms) => {
                uniforms.model = model;
                self.bind_matrix("model", &model);
                let normal_matrix = model.invert().unwrap().transpose();
                self.bind_matrix("normalMatrix", &normal_matrix);
                Ok(())
            }
            _ => return Err(String::from("No uniform buffer object found!")),
        }
    }
    pub fn render(&self) -> Result<(), JsValue> {
        self.ctx.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);
        // log!("Num vertices: {}", self.num_triangles);

        self.ctx.draw_elements_with_i32(
            GL::TRIANGLES,
            self.num_triangles as i32,
            GL::UNSIGNED_SHORT,
            0,
        );
        Ok(())
    }
    pub fn resize(&mut self) {
        self.aspect_ratio = Self::resize_canvas(&mut self.canvas);
        use cgmath::{perspective, Deg, Rad};
        self.uniforms.as_mut().unwrap().proj =
            perspective(Rad::from(Deg(60.)), self.aspect_ratio(), 0.1, 100.);
        self.ctx.viewport(
            0,
            0,
            self.canvas.width() as i32,
            self.canvas.height() as i32,
        );
        self.bind_uniforms().expect("Can't bind uniforms");
    }
    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }
}
