#version 300 es
in vec3 position;

uniform mat4 model, view, proj;
uniform vec4 color;

out vec4 f_color;

void main() {
	gl_Position = proj * view * model * vec4(position, 1.0);
	f_color = color;
}
