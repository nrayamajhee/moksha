use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext as GL;

mod renderer;
mod shader;

pub use renderer::{Config, DrawMode, Renderer};

pub use shader::{
    bind_attribute, bind_buffer_and_attribute, bind_buffer_f32, bind_index_buffer, bind_texture,
    bind_uniform_i32, bind_uniform_mat4, bind_uniform_vec4, compile_shader, create_color_program,
    create_program, create_texture_program, create_vertex_color_program, link_program, ShaderType,
};
