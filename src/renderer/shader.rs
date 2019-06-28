use crate::dom_factory::add_event;
use cgmath::Matrix4;
use js_sys::{Float32Array, Uint16Array, Uint8Array};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::{HtmlImageElement, WebGlProgram, WebGlRenderingContext as GL, WebGlShader, WebGlTexture};

#[derive(Copy, Clone, Debug)]
pub enum ShaderType {
    Color,
    VertexColor,
    Texture,
}

pub fn create_program(gl: &GL, vertex: &str, fragment: &str) -> Result<WebGlProgram, String> {
    let vert_shader = compile_shader(gl, GL::VERTEX_SHADER, vertex)?;
    let frag_shader = compile_shader(gl, GL::FRAGMENT_SHADER, fragment)?;
    let program = link_program(gl, &vert_shader, &frag_shader, true)?;
    Ok(program)
}

pub fn color_program(gl: &GL) -> Result<WebGlProgram, String> {
    let shader = create_program(
        gl,
        r#"
            attribute vec4 position;
			attribute vec3  normal;

            uniform mat4 model, view, proj, normalMatrix;
            uniform lowp vec4 color;

			varying lowp vec3 lighting;
            varying lowp vec4 f_color;

            void main() {
                gl_Position = proj * view * model * position;
                f_color = color;

				highp vec3 ambientLight = vec3(0.1, 0.1, 0.1);
				highp vec3 directionalLightColor = vec3(1, 1, 1);
				highp vec3 directionalVector = normalize(vec3(5.0, 5.0, 0.0));

				highp vec4 transformedNormal = normalMatrix * vec4(normal, 1.0);

				highp float directional = max(dot(transformedNormal.xyz, directionalVector), 0.0);
				lighting = ambientLight + (directionalLightColor * directional);
            }
        "#,
        r#"
			varying lowp vec3 lighting;
            varying lowp vec4 f_color;

            void main() {
				gl_FragColor = vec4(f_color.xyz * lighting, 1.0);
            }
        "#,
    )?;
    Ok(shader)
}

pub fn vertex_color_program(gl: &GL) -> Result<WebGlProgram, String> {
    let shader = create_program(
        gl,
        r#"
            attribute vec4 position;
			attribute vec3  normal;
            attribute vec4 color;

            uniform mat4 model, view, proj, normalMatrix;

            varying lowp vec4 f_color;
			varying lowp vec3 lighting;

            void main() {
                gl_Position = proj * view * model * position;
                f_color = color;

				highp vec3 ambientLight = vec3(0.1, 0.1, 0.1);
				highp vec3 directionalLightColor = vec3(1, 1, 1);
				highp vec3 directionalVector = normalize(vec3(5.0, 5.0, 0.0));

				highp vec4 transformedNormal = normalMatrix * vec4(normal, 1.0);

				highp float directional = max(dot(transformedNormal.xyz, directionalVector), 0.0);
				lighting = ambientLight + (directionalLightColor * directional);
            }
        "#,
        r#"
            varying lowp vec4 f_color;
			varying lowp vec3 lighting;

            void main() {
				gl_FragColor =vec4(f_color.xyz * lighting, 1.0);
            }
        "#,
    )?;
    Ok(shader)
}

pub fn texture_program(gl: &GL) -> Result<WebGlProgram, String> {
    let shader = create_program(
        gl,
        r#"
            attribute vec4 position;
			attribute vec3 normal;
            attribute vec2 texCoord;

            uniform mat4 model, view, proj, normalMatrix;

            varying lowp vec2 f_texCoord;
			varying lowp vec3 lighting;

            void main() {
                gl_Position = proj * view * model * position;
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
            varying lowp vec2 f_texCoord;
			varying lowp vec3 lighting;

			uniform sampler2D sampler;

            void main() {
				highp vec4 texelColor = texture2D(sampler, f_texCoord);
				gl_FragColor = vec4(texelColor.rgb * lighting, texelColor.a);
            }
        "#,
    )?;
    Ok(shader)
}

pub fn compile_shader(gl: &GL, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, GL::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}
pub fn link_program(
    gl: &GL,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
    validate: bool,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, GL::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        if validate {
            gl.validate_program(&program);
            if (gl.get_program_parameter(&program, GL::VALIDATE_STATUS))
                .as_bool()
                .unwrap_or(false)
            {
                Ok(program)
            } else {
                Err(gl
                    .get_program_info_log(&program)
                    .unwrap_or_else(|| String::from("Unknown error creating program object")))
            }
        } else {
            Ok(program)
        }
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

fn is_power_of_2(val: u32) -> bool {
    return (val & (val - 1)) == 0;
}

pub fn bind_texture(gl: &GL, url: &str) -> Result<(), JsValue> {
    let texture = gl.create_texture().expect("Can't create texture!");
    gl.bind_texture(GL::TEXTURE_2D, Some(&texture));
    let pixel = unsafe { Uint8Array::view(&[255, 0, 255, 255]) };
    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
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
    let gl = gl.clone();
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

pub fn bind_matrix(gl: &GL, program: &WebGlProgram, attribute: &str, matrix: &Matrix4<f32>) {
    let mat: &[f32; 16] = matrix.as_ref();
    let mat_attrib = gl
        .get_uniform_location(program, attribute)
        .expect(format!("Can't bind uniform: {}", attribute).as_str());
    gl.uniform_matrix4fv_with_f32_array(Some(&mat_attrib), false, mat);
}

pub fn bind_vector(gl: &GL, program: &WebGlProgram, attribute: &str, vector: &[f32]) {
    let mat_attrib = gl
        .get_uniform_location(program, attribute)
        .expect(format!("Can't bind uniform: {}", attribute).as_str());
    gl.uniform4f(
        Some(&mat_attrib),
        vector[0],
        vector[1],
        vector[2],
        vector[3],
    );
}

pub fn bind_buffer_f32(gl: &GL, data: &[f32]) -> Result<(), JsValue> {
    let buffer = gl.create_buffer().ok_or("failed to create buffer")?;
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));
    let buffer_array = unsafe { Float32Array::view(&data) };
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &buffer_array, GL::STATIC_DRAW);
    Ok(())
}
pub fn bind_index_buffer(gl: &GL, data: &[u16]) -> Result<(), JsValue> {
    let buffer = gl.create_buffer().ok_or("failed to create buffer")?;
    gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&buffer));
    let buffer_array = unsafe { Uint16Array::view(&data) };
    gl.buffer_data_with_array_buffer_view(GL::ELEMENT_ARRAY_BUFFER, &buffer_array, GL::STATIC_DRAW);
    Ok(())
}
pub fn bind_attribute(gl: &GL, program: &WebGlProgram, name: &str, size: i32) {
    let attribute = gl.get_attrib_location(program, name);
    gl.vertex_attrib_pointer_with_i32(attribute as u32, size, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(attribute as u32);
}

pub fn bind_uniform_1i(gl: &GL, program: &WebGlProgram, name: &str, value: i32) {
    let attrib = gl
        .get_uniform_location(program, name)
        .expect(format!("Can't bind uniform: {}", name).as_str());
    gl.uniform1i(Some(&attrib), value);
}
