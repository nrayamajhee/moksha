#version 300 es
// Thanks to Florian Boesh for his tutorial on how to achieve fancy wireframe with
// barycentric coordinates. Please refer to the following url for further details:
// <http://codeflow.org/entries/2012/aug/02/easy-wireframe-display-with-barycentric-coordinates/>
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
