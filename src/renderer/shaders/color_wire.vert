#version 300 es
// Thanks to Florian Boesh for his tutorial on how to achieve fancy wireframe with
// barycentric coordinates. Please refer to the following url for further details:
// <http://codeflow.org/entries/2012/aug/02/easy-wireframe-display-with-barycentric-coordinates/>
uniform mat4 model, view, proj;
uniform vec3 eye;

in vec3 position, normal, barycentric;
out vec3 surface_normal, object_pos, view_dir, frag_bc;

void main() {
	object_pos = vec3(model * vec4(position, 1.0));
	gl_Position = proj * view * vec4(object_pos, 1.0);
	surface_normal = mat3(transpose(inverse(model))) * normal;
	view_dir = normalize(eye - object_pos);
	frag_bc = barycentric;
}
