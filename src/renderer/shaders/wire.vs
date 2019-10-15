#version 300 es
in vec3 position, barycentric;

uniform mat4 model, view, proj;
uniform vec4 color;

out vec4 f_color;
out vec3 frag_bc;

void main() {
	gl_Position = proj * view * model * vec4(position, 1.0);
	gl_PointSize = 10.0;
	f_color = color;
	frag_bc = barycentric;
}
